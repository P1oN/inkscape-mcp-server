"""Allowlisted subtree replacement with stable container ID and explicit reference policy."""

from __future__ import annotations

import copy
from typing import Literal

from lxml import etree

from inkscape_mcp.edit.compose import parse_and_scrub
from inkscape_mcp.edit.dom import SAFE_ID_RE, EditError, all_ids, require_target
from inkscape_mcp.edit.pipeline import MutateFn
from inkscape_mcp.edit.structure import references, reject_stylesheets
from inkscape_mcp.workspace.limits import LimitExceeded, check_input_bytes_size

ReferencePolicy = Literal["reject_changes", "allow_retained"]


def make_replace_fragment(
    object_id: str, svg: str, reference_policy: ReferencePolicy = "reject_changes"
) -> MutateFn:
    if reference_policy not in {"reject_changes", "allow_retained"}:
        raise EditError("invalid external reference policy")
    try:
        check_input_bytes_size(svg.encode())
    except LimitExceeded as exc:
        raise EditError("fragment exceeds the configured input size limit") from exc
    replacement = parse_and_scrub(svg)
    incoming_id = replacement.get("id")
    if incoming_id is not None and incoming_id != object_id:
        raise EditError("fragment root ID must match selected container ID (or be omitted)")
    replacement.set("id", object_id)
    new_ids = [n.get("id") for n in replacement.iter() if n.get("id")]
    if len(new_ids) != len(set(new_ids)):
        raise EditError("duplicate IDs inside fragment")
    if any(not SAFE_ID_RE.fullmatch(oid) for oid in new_ids if oid is not None):
        raise EditError("fragment contains an invalid ID")

    def mutate(tree: etree._ElementTree) -> str:
        root = tree.getroot()
        target = require_target(root, object_id)
        parent = target.getparent()
        if parent is None:
            raise EditError("use set_document_svg for the document root")
        if target.tag != replacement.tag:
            raise EditError("replacement must keep the selected element's qualified tag")
        reject_stylesheets(root)
        old_subtree = set(target.iter())
        old_ids = all_ids(target)
        outside_ids = {n.get("id") for n in root.iter() if n not in old_subtree and n.get("id")}
        if set(new_ids) & outside_ids:
            raise EditError("fragment ID conflicts with the rest of the document")
        known = outside_ids | set(new_ids)
        if any(references(n) - known for n in replacement.iter()):
            raise EditError("fragment contains unresolved references")
        removed = old_ids - set(new_ids)
        external = set().union(*(references(n) for n in root.iter() if n not in old_subtree))
        if external & removed:
            raise EditError("rest of document references IDs removed by fragment replacement")
        candidate = copy.deepcopy(replacement)
        candidate.tail = target.tail
        if etree.tostring(target, method="c14n", exclusive=True) == etree.tostring(
            candidate, method="c14n", exclusive=True
        ):
            return "fragment already matches"
        if reference_policy == "reject_changes" and external & old_ids:
            raise EditError(
                "retained subtree IDs are externally referenced; "
                "explicitly allow_retained to update their appearance"
            )
        parent.replace(target, candidate)
        return (
            f"replaced fragment {object_id!r}; retained {len(old_ids & set(new_ids))} IDs, "
            f"removed {len(removed)}; external reference policy {reference_policy}"
        )

    return mutate
