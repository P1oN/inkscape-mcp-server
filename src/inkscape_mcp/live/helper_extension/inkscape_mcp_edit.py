"""Fixed semantic edits, planned on a copy before publishing one native Undo transaction.

Runs with vendor inkex in the one-shot extension, never inside the MCP server.
"""
from copy import deepcopy
import math
import re

import inkex

SVG = "http://www.w3.org/2000/svg"
INKSCAPE = "http://www.inkscape.org/namespaces/inkscape"
SODIPODI = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"
OPS = {"style", "text", "duplicate", "delete", "group", "ungroup", "raise", "lower", "front", "back"}
DRAWABLE = {"g", "rect", "circle", "ellipse", "path", "line", "polygon", "polyline", "text", "use", "image"}


def local(node):
    return node.tag.rsplit("}", 1)[-1] if isinstance(node.tag, str) else ""


def locked(node):
    return any(e.get(f"{{{SODIPODI}}}insensitive") == "true" for e in [node, *node.iterancestors()])


def references(root, ids, excluded):
    # Conservative refusal includes CSS selectors and Inkscape connector references.
    for node in root.iter():
        if node in excluded:
            continue
        values = [*node.attrib.values()]
        if local(node) == "style":
            values.append(node.text or "")
        if any(re.search(r"#" + re.escape(oid) + r"(?![\w.:-])", value)
               for oid in ids for value in values):
            raise ValueError("selected objects are referenced; detach references before deleting")


def plan_edit(svg, request):
    if not re.fullmatch(r"mcp_[0-9a-f]{32}", request["nonce"]):
        raise ValueError("invalid edit identity")
    operation = request["operation"]
    if operation not in OPS:
        raise ValueError("unsupported edit operation")
    ids = request["selection"]
    if not isinstance(ids, list) or not ids or len(ids) > 10000 or len(set(ids)) != len(ids):
        raise ValueError("invalid selection")
    root = deepcopy(svg)
    identified = [node for node in root.iter() if node.get("id")]
    by_id = {node.get("id"): node for node in identified}
    if len(by_id) != len(identified):
        raise ValueError("document contains duplicate ids")
    if request["nonce"] in by_id:
        raise ValueError("edit identity collision")
    nodes = [by_id[oid] for oid in ids]
    for node in nodes:
        if local(node) not in DRAWABLE or node is root or node.getparent() is None:
            raise ValueError("select drawable objects")
        if node.get(f"{{{INKSCAPE}}}groupmode") == "layer":
            raise ValueError("select objects inside the layer, not the layer itself")
        if any(local(a) in {"defs", "clipPath", "mask", "pattern", "symbol"} for a in node.iterancestors()):
            raise ValueError("definitions are not editable selections")
        if any(locked(e) for e in node.iter()):
            raise ValueError("selection contains locked objects or belongs to a locked layer")
    # Parent + child selections must not transform/delete descendants twice.
    nodes = [node for node in nodes if not any(a in nodes for a in node.iterancestors())]
    affected = [node.get("id") for node in nodes]
    if operation in {"duplicate", "group", "ungroup"} and any(local(e) == "style" for e in root.iter()):
        raise ValueError("structural edits with stylesheets require inline styles first")

    if operation == "style":
        style = request.get("style", {})
        transform = request.get("transform")
        if set(style) - {"fill", "stroke", "stroke-width", "opacity"} or not (style or transform):
            raise ValueError("invalid style edit")
        if any(not isinstance(v, str) or any(c in v for c in ";{}\\\n") for v in style.values()):
            raise ValueError("invalid style value")
        for node in nodes:
            node.style.update(style)
            if transform:
                if any(local(a) == "svg" and a is not root for a in node.iterancestors()) or any(
                    "transform" in a.style for a in [node, *node.iterancestors()]
                ):
                    raise ValueError("CSS transforms and nested SVG viewports are unsupported")
                parent = node.getparent().composed_transform()
                # Conjugate document-coordinate delta into the object's parent space.
                delta = inkex.Transform(transform)
                result = -parent @ delta @ parent @ node.transform
                if not all(math.isfinite(v) for row in result.matrix for v in row):
                    raise ValueError("non-finite transform")
                node.transform = result
    elif operation == "text":
        if len(nodes) != 1 or local(nodes[0]) != "text":
            raise ValueError("select exactly one text object")
        text = request["text"]
        if not isinstance(text, str) or len(text) > 100000 or any(ord(c) < 32 for c in text):
            raise ValueError("replacement must be a single line of text")
        node = nodes[0]
        if any(local(e) not in {"text", "tspan"} for e in node.iter()):
            raise ValueError("text paths and flowed text are unsupported")
        leaves = [e for e in node.iter() if len(e) == 0]
        if len(leaves) != 1 or any((e.text or "").strip() for e in node.iter() if len(e)) or any((e.tail or "").strip() for e in node.iter()):
            raise ValueError("mixed formatting or multiple text runs; select simple single-run text")
        leaves[0].text = text
    elif operation == "duplicate":
        clones = [deepcopy(node) for node in nodes]
        remap = {}
        for clone in clones:
            for elem in clone.iter():
                if elem.get("id"):
                    remap[elem.get("id")] = f"{request['nonce']}_{len(remap)}"
        for node, clone in zip(nodes, clones):
            for elem in clone.iter():
                for key, value in list(elem.attrib.items()):
                    if key == "id":
                        elem.set(key, remap[value])
                    else:
                        value = re.sub(r"url\(\s*(['\"]?)#([\w.:-]+)\1\s*\)",
                                       lambda m: f"url(#{remap.get(m[2], m[2])})", value)
                        if value.startswith("#") and (key.endswith("href") or key.endswith("connector-start") or key.endswith("connector-end")):
                            value = "#" + remap.get(value[1:], value[1:])
                        elem.set(key, value)
            node.addnext(clone)
        affected = list(remap.values())
    elif operation == "delete":
        excluded = {e for node in nodes for e in node.iter()}
        references(root, {e.get("id") for e in excluded if e.get("id")}, excluded)
        for node in nodes:
            node.getparent().remove(node)
    elif operation == "group":
        parent = nodes[0].getparent()
        if any(node.getparent() is not parent for node in nodes):
            raise ValueError("group requires objects in the same layer or parent")
        nodes.sort(key=parent.index)
        positions = [parent.index(node) for node in nodes]
        if positions != list(range(positions[0], positions[0] + len(nodes))):
            raise ValueError("group requires consecutive siblings to preserve stacking")
        group = inkex.Group(id=request["nonce"])
        parent.insert(positions[0], group)
        for node in nodes:
            group.append(node)
        affected = [request["nonce"], *affected]
    elif operation == "ungroup":
        # Removing arbitrary group effects can change compositing. Refuse those rather
        # than flatten masks, opacity, inherited styles or references incorrectly.
        for node in nodes:
            if local(node) != "g" or any(k not in {"id", "transform", f"{{{INKSCAPE}}}label"} for k in node.attrib):
                raise ValueError("ungroup supports plain groups without inherited style or effects")
        references(root, set(affected), set(nodes))
        affected = []
        for node in nodes:
            parent = node.getparent()
            index = parent.index(node)
            for child in list(node):
                if hasattr(child, "transform"):
                    child.transform = node.transform @ child.transform
                parent.insert(index, child)
                index += 1
                if child.get("id"):
                    affected.append(child.get("id"))
            parent.remove(node)
    else:
        parents = {node.getparent() for node in nodes}
        chosen = set(nodes)
        for parent in parents:
            ordered = [e for e in parent if e in chosen]
            if operation in {"front", "back"}:
                siblings = [e for e in parent if local(e) in DRAWABLE]
                anchor = siblings[-1] if operation == "front" else siblings[0]
                # Stable relative order within the selection and no cross-layer moves.
                if operation == "front":
                    for node in ordered:
                        if node is not anchor:
                            anchor.addnext(node)
                        anchor = node
                else:
                    for node in reversed(ordered):
                        if node is not anchor:
                            anchor.addprevious(node)
                        anchor = node
            else:
                for node in reversed(ordered) if operation == "raise" else ordered:
                    sibling = node.getnext() if operation == "raise" else node.getprevious()
                    while sibling is not None and local(sibling) not in DRAWABLE:
                        sibling = sibling.getnext() if operation == "raise" else sibling.getprevious()
                    if sibling is not None and sibling not in chosen:
                        sibling.addnext(node) if operation == "raise" else sibling.addprevious(node)
    return root, affected
