"""One-shot effect: all inserted nodes share Inkscape's normal extension Undo transaction."""
import json
import os
from pathlib import Path

import inkex
from lxml import etree
from inkscape_mcp_insert_payload import document_fingerprint, prepare_fragment


class InsertOnce(inkex.EffectExtension):
    def effect(self):
        directory = os.environ.get("INKSCAPE_MCP_MANAGED_DIR")
        if not directory:
            raise inkex.AbortExtension("Start a managed MCP session before inserting.")
        root = Path(directory)
        request_path = root / "insert-request.json"
        result = {"nonce": None, "ok": False}
        try:
            if request_path.stat().st_size > 2 * 1024 * 1024:
                raise inkex.AbortExtension("Insertion request is too large.")
            request = json.loads(request_path.read_text())
            nonce = request["nonce"]
            result["nonce"] = nonce
            existing = {elem.get("id") for elem in self.svg.iter() if elem.get("id")}
            if existing != set(request["expected_ids"]):
                raise ValueError("document changed before insertion")
            if document_fingerprint(self.svg) != request["expected_fingerprint"]:
                raise ValueError("drawing content changed before insertion")
            if request.get("operation"):
                from inkscape_mcp_edit import plan_edit
                # The extension input selection is compared with the captured GUI selection.
                if {e.get("id") for e in self.svg.selection} != set(request["selection"]):
                    raise ValueError("selection changed before edit")
                working, ids = plan_edit(self.svg, request)
                fingerprint = document_fingerprint(working)
                self.document._setroot(working)
                result.update(ok=True, ids=ids, fingerprint=fingerprint)
                return
            payload, ids = prepare_fragment(request["fragment"], nonce)
            if existing.intersection(ids):
                raise ValueError("insertion id collision")
            group = etree.fromstring(payload)
            # Append to the root in document coordinates; never inherit the current layer transform.
            self.svg.append(group)
            result.update(ok=True, ids=ids)
        except Exception as exc:
            result["error"] = type(exc).__name__
            if "request" in locals() and request.get("operation"):
                # Return the unchanged SVG instead of opening a blocking native error dialog.
                from inkscape_mcp_edit_errors import EDIT_REFUSALS
                result["error"] = str(exc) if str(exc) in EDIT_REFUSALS else "invalid document or selection"
                return
            raise inkex.AbortExtension("Insertion refused; document context or SVG is invalid.") from exc
        finally:
            pending = root / "insert-result.tmp"
            pending.write_text(json.dumps(result))
            pending.chmod(0o600)
            pending.replace(root / "insert-result.json")


if __name__ == "__main__":
    InsertOnce().run()
