"""Bounded, self-contained SVG insertion payload; also copied beside the inkex helper."""

from __future__ import annotations

import hashlib
import json
import re

from lxml import etree

SVG = "http://www.w3.org/2000/svg"
MAX_BYTES = 1024 * 1024
ALLOWED = frozenset(
    {
        "svg",
        "g",
        "defs",
        "rect",
        "circle",
        "ellipse",
        "path",
        "line",
        "polygon",
        "polyline",
        "text",
        "tspan",
        "use",
        "linearGradient",
        "radialGradient",
        "stop",
        "clipPath",
        "mask",
        "pattern",
        "title",
        "desc",
    }
)
URL = re.compile(r"url\(\s*['\"]?(#[\w.:-]+)['\"]?\s*\)", re.IGNORECASE)


def document_fingerprint(root: etree._Element) -> str:
    """Compare drawing content, ignoring namespace prefixes and UI-only metadata."""
    records = []
    for elem in root.iter():
        if not isinstance(elem.tag, str):
            continue
        if any(
            etree.QName(e).localname in {"metadata", "namedview"}
            for e in [elem, *elem.iterancestors()]
        ):
            continue
        attrs = sorted(
            (key, value)
            for key, value in elem.attrib.items()
            if elem is not root
            or (etree.QName(key).namespace is None and key not in {"id", "version"})
        )
        records.append(
            (
                len(list(elem.iterancestors())),
                elem.tag,
                attrs,
                (elem.text or "").strip(),
                (elem.tail or "").strip(),
            )
        )
    return hashlib.sha256(json.dumps(records, ensure_ascii=False).encode()).hexdigest()


def prepare_fragment(fragment: str, prefix: str) -> tuple[bytes, list[str]]:
    """Remap ids and local references, reject executable/external content, wrap as one group."""
    if not re.fullmatch(r"mcp_[0-9a-f]{32}", prefix):
        raise ValueError("invalid insertion id")
    if not fragment.strip() or len(fragment.encode()) > MAX_BYTES:
        raise ValueError("insertion fragment is empty or too large")
    parser = etree.XMLParser(resolve_entities=False, no_network=True, load_dtd=False)
    try:
        root = etree.fromstring(f'<svg xmlns="{SVG}">{fragment}</svg>'.encode(), parser)
    except etree.XMLSyntaxError as exc:
        raise ValueError("malformed insertion XML") from exc
    if sum(1 for _ in root.iter()) > 10000:
        raise ValueError("too many insertion elements")
    ids: dict[str, str] = {}
    for elem in root.iter():
        if not isinstance(elem.tag, str):
            raise ValueError("insertion contains non-element content")
        element_name = etree.QName(elem)
        if element_name.namespace != SVG or element_name.localname not in ALLOWED:
            raise ValueError("unsupported SVG insertion element")
        if elem.get("id"):
            old = elem.get("id", "")
            if old in ids:
                raise ValueError("duplicate insertion id")
            ids[old] = f"{prefix}_{len(ids)}"
    group = etree.Element(f"{{{SVG}}}g", nsmap={"svg": SVG}, id=prefix)
    for elem in root.iter():
        for key, value in list(elem.attrib.items()):
            name = etree.QName(key).localname
            if name.lower().startswith("on") or name == "base":
                raise ValueError("event attributes are unsupported")
            if name == "id":
                elem.set(key, ids[value])
            elif name == "href":
                if not value.startswith("#") or value[1:] not in ids:
                    raise ValueError("href must reference an id in the fragment")
                elem.set(key, "#" + ids[value[1:]])
            elif "url" in value.lower() or name == "style":
                if "\\" in value or "@" in value:
                    raise ValueError("unsupported CSS insertion value")

                def replace(match: re.Match[str]) -> str:
                    target = match[1][1:]
                    if target not in ids:
                        raise ValueError("paint reference must be internal to the fragment")
                    return f"url(#{ids[target]})"

                rewritten = URL.sub(replace, value)
                if "url" in URL.sub("", value).lower():
                    raise ValueError("external or malformed CSS URL")
                elem.set(key, rewritten)
    for child in list(root):
        group.append(child)
    if not len(group):
        raise ValueError("insertion has no objects")
    return etree.tostring(group), [prefix, *ids.values()]
