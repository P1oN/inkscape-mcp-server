"""Conservative structural preservation checks and affine coordinate compensation."""

from __future__ import annotations

import math
import re

from lxml import etree

from inkscape_mcp.document.inspect import INKSCAPE_NS
from inkscape_mcp.edit.dom import EditError, require_target
from inkscape_mcp.edit.pipeline import MutateFn

Matrix = tuple[float, float, float, float, float, float]
IDENTITY: Matrix = (1, 0, 0, 1, 0, 0)
LABEL = f"{{{INKSCAPE_NS}}}label"
MODE = f"{{{INKSCAPE_NS}}}groupmode"
_NUMBER = re.compile(r"[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?")
_CALL = re.compile(r"(matrix|translate|scale|rotate|skewX|skewY)\s*\(([^()]*)\)")


def multiply(left: Matrix, right: Matrix) -> Matrix:
    a, b, c, d, e, f = left
    g, h, i, j, k, offset_y = right
    result = (
        a * g + c * h,
        b * g + d * h,
        a * i + c * j,
        b * i + d * j,
        a * k + c * offset_y + e,
        b * k + d * offset_y + f,
    )
    if not all(math.isfinite(v) for v in result):
        raise EditError("non-finite composed transform")
    return result


def inverse(matrix: Matrix) -> Matrix:
    a, b, c, d, e, f = matrix
    determinant = a * d - b * c
    if not math.isfinite(determinant) or abs(determinant) < 1e-12:
        raise EditError("singular or ill-conditioned parent transform")
    return (
        d / determinant,
        -b / determinant,
        -c / determinant,
        a / determinant,
        (c * f - d * e) / determinant,
        (b * e - a * f) / determinant,
    )


def parse_transform(raw: str) -> Matrix:
    result = IDENTITY
    cursor = 0
    for match in _CALL.finditer(raw):
        if raw[cursor : match.start()].strip(" ,\t\r\n"):
            raise EditError("unsupported SVG transform")
        body = match[2]
        if _NUMBER.sub("", body).strip(" ,\t\r\n"):
            raise EditError("invalid SVG transform arguments")
        args = [float(v) for v in _NUMBER.findall(body)]
        if not all(math.isfinite(v) for v in args):
            raise EditError("non-finite SVG transform")
        name = match[1]
        matrix = IDENTITY
        if name == "matrix" and len(args) == 6:
            matrix = (args[0], args[1], args[2], args[3], args[4], args[5])
        elif name == "translate" and len(args) in (1, 2):
            matrix = (1, 0, 0, 1, args[0], args[1] if len(args) == 2 else 0)
        elif name == "scale" and len(args) in (1, 2):
            matrix = (args[0], 0, 0, args[-1], 0, 0)
        elif name == "rotate" and len(args) in (1, 3):
            angle = math.radians(args[0])
            cosine, sine = math.cos(angle), math.sin(angle)
            matrix = (cosine, sine, -sine, cosine, 0, 0)
            if len(args) == 3:
                x, y = args[1:]
                matrix = multiply(multiply((1, 0, 0, 1, x, y), matrix), (1, 0, 0, 1, -x, -y))
        elif name in ("skewX", "skewY") and len(args) == 1:
            tangent = math.tan(math.radians(args[0]))
            matrix = (1, 0, tangent, 1, 0, 0) if name == "skewX" else (1, tangent, 0, 1, 0, 0)
        else:
            raise EditError("unsupported SVG transform arity")
        result = multiply(result, matrix)
        cursor = match.end()
    if raw[cursor:].strip():
        raise EditError("unsupported SVG transform")
    return result


def matrix_text(matrix: Matrix) -> str:
    if not all(math.isfinite(v) for v in matrix):
        raise EditError("non-finite matrix")
    return "matrix(" + ",".join(format(v, ".17g") for v in matrix) + ")"


def local(node: etree._Element) -> str:
    return etree.QName(node).localname if isinstance(node.tag, str) else ""


def composed(node: etree._Element | None) -> Matrix:
    if node is None:
        return IDENTITY
    chain = [*reversed(list(node.iterancestors())), node]
    result = IDENTITY
    for ancestor in chain:
        if local(ancestor) == "svg" and ancestor.getparent() is not None:
            raise EditError("nested SVG viewports require preparation")
        if "transform" in ancestor.get("style", "").lower():
            raise EditError("CSS transforms require preparation")
        result = multiply(result, parse_transform(ancestor.get("transform", "")))
    return result


def references(node: etree._Element) -> set[str]:
    refs: set[str] = set()
    for key, raw in node.attrib.items():
        value = raw if isinstance(raw, str) else raw.decode()
        if etree.QName(key).localname in {"href", "connection-start", "connection-end"}:
            if value.startswith("#"):
                refs.add(value[1:])
        refs.update(re.findall(r"url\(\s*['\"]?#([^\s)'\"]+)", value))
    return refs


def reject_stylesheets(root: etree._Element) -> None:
    if any(local(node) == "style" for node in root.iter()):
        raise EditError("structural preservation with stylesheets requires inline styles first")


def make_group_mode(object_id: str, mode: str) -> MutateFn:
    if mode not in {"group", "layer"}:
        raise EditError("mode must be group or layer")

    def mutate(tree: etree._ElementTree) -> str:
        root = tree.getroot()
        node = require_target(root, object_id)
        if local(node) != "g":
            raise EditError("group/layer conversion requires a g container")
        if (node.get(MODE) == "layer") == (mode == "layer"):
            return "container mode already matches"
        reject_stylesheets(root)
        if mode == "layer":
            node.set(MODE, "layer")
        else:
            if MODE in node.attrib:
                del node.attrib[MODE]
        return f"converted {object_id!r} to {mode}"

    return mutate


def paint_order(root: etree._Element) -> list[etree._Element]:
    result: list[etree._Element] = []

    def visit(node: etree._Element) -> None:
        tag = local(node)
        if tag in {"svg", "g"}:
            for child in node:
                visit(child)
        elif tag not in {"defs", "metadata", "title", "desc", "namedview", ""}:
            result.append(node)

    visit(root)
    return result


def reparent_preserving(tree: etree._ElementTree, object_id: str, new_parent_id: str) -> str:
    root = tree.getroot()
    node = require_target(root, object_id)
    dest = require_target(root, new_parent_id)
    old = node.getparent()
    if old is None or dest is node or dest in node.iterdescendants():
        raise EditError("cannot reparent root or create a container cycle")
    if local(dest) != "g":
        raise EditError("destination must be a group/layer")
    if old is dest:
        return "object already belongs to destination"
    reject_stylesheets(root)
    old_chain = [old, *old.iterancestors()]
    new_chain = [dest, *dest.iterancestors()]
    divergent = set(old_chain) ^ set(new_chain)
    allowed = {"id", "transform", LABEL, MODE}
    for ancestor in divergent:
        if local(ancestor) != "g" or set(ancestor.attrib) - allowed:
            raise EditError(
                "changed ancestors carry style, effects, locks or viewport metadata; "
                "prepare plain groups first"
            )
    ids = {n.get("id") for n in node.iter() if n.get("id")}
    ids.update(n.get("id") for n in divergent if n.get("id"))
    subtree = set(node.iter())
    if any(references(n) & ids for n in root.iter() if n not in subtree):
        raise EditError("external references to moved subtree may change appearance")
    composed(node)  # also reject CSS transforms and nested viewports on the moved object
    old_matrix, new_matrix = composed(old), composed(dest)
    inverse(old_matrix)
    transform = multiply(
        multiply(inverse(new_matrix), old_matrix), parse_transform(node.get("transform", ""))
    )
    before = paint_order(root)
    # Work on the pipeline's disposable tree: rejection is always before disk mutation.
    dest.append(node)
    if before != paint_order(root):
        raise EditError("move changes global paint order; overlap preservation is ambiguous")
    node.set("transform", matrix_text(transform))
    return f"reparented {object_id!r}, preserving document transform and paint order"
