"""Read-only structural observations and optional editing recommendations."""

from __future__ import annotations

from typing import Literal

from lxml import etree
from pydantic import BaseModel, Field

from inkscape_mcp.edit.structure import LABEL, MODE, local


class EditabilityOptions(BaseModel):
    enabled: bool = True
    check_labels: bool = True
    semantic_group_ids: list[str] = Field(default_factory=list, max_length=200)
    layer_advisory_threshold: int = Field(default=12, ge=1, le=10000)
    max_group_depth: int = Field(default=6, ge=1, le=100)
    fragmentation_threshold: int = Field(default=30, ge=1, le=10000)


class StructureAdvice(BaseModel):
    code: str
    kind: Literal["observation", "recommendation"]
    object_id: str | None = None
    message: str


class EditabilityReport(BaseModel):
    group_count: int
    layer_count: int
    max_group_depth: int
    single_child_groups: int
    advice: list[StructureAdvice]
    truncated: bool = False
    note: str = "Advice is optional; names/counts cannot establish semantics or tracing."


def analyze_editability(root: etree._Element, options: EditabilityOptions) -> EditabilityReport:
    groups = [node for node in root.iter() if local(node) == "g"]
    layers = [node for node in groups if node.get(MODE) == "layer"]
    depths = [sum(local(a) == "g" for a in node.iterancestors()) + 1 for node in groups]
    fragments = [node for node in groups if len(node) == 1]
    advice: list[StructureAdvice] = []

    def add(
        code: str,
        kind: Literal["observation", "recommendation"],
        message: str,
        node: etree._Element | None = None,
    ) -> None:
        advice.append(
            StructureAdvice(
                code=code,
                kind=kind,
                message=message,
                object_id=node.get("id") if node is not None else None,
            )
        )

    if options.enabled:
        semantic = set(options.semantic_group_ids)
        for node, depth in zip(groups, depths, strict=True):
            if options.check_labels and not (node.get(LABEL) or "").strip():
                if not semantic or node.get("id") in semantic:
                    add(
                        "missing_group_label",
                        "recommendation",
                        "Consider a readable label if this group represents a selectable object.",
                        node,
                    )
            if node.get("id") in semantic and node.get(MODE) == "layer":
                add(
                    "semantic_layer",
                    "recommendation",
                    "Designated semantic object is a layer; consider ordinary group selection.",
                    node,
                )
            if depth > options.max_group_depth:
                add(
                    "deep_groups",
                    "recommendation",
                    "Consider simplifying this nesting if unnecessary.",
                    node,
                )
            if any(
                key in node.attrib for key in ("style", "opacity", "filter", "mask", "clip-path")
            ):
                add(
                    "structural_style_risk",
                    "observation",
                    "Container has effects; moving children may change inheritance or compositing.",
                    node,
                )
        if len(layers) > options.layer_advisory_threshold:
            add(
                "many_layers",
                "recommendation",
                "Layer count exceeds your threshold; check whether organization is intentional.",
            )
        if len(fragments) > options.fragmentation_threshold:
            add(
                "fragmentation",
                "recommendation",
                "Many groups have one child; review wrapper structure if selection is cumbersome.",
            )
        if any(local(node) == "style" for node in root.iter()):
            add(
                "stylesheet_structure_risk",
                "observation",
                "Stylesheet selectors may depend on ancestry, IDs or container metadata.",
            )
        add(
            "paint_order_risk",
            "observation",
            "Reparenting can change paint order; preservation cannot be inferred from group names.",
        )
    return EditabilityReport(
        group_count=len(groups),
        layer_count=len(layers),
        max_group_depth=max(depths, default=0),
        single_child_groups=len(fragments),
        advice=advice[:200],
        truncated=len(advice) > 200,
    )
