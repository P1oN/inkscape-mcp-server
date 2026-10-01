"""Public, path-free refusal reasons shared with the one-shot extension."""

EDIT_REFUSALS = frozenset(
    {
        "CSS transforms and nested SVG viewports are unsupported",
        "definitions are not editable selections",
        "document changed before insertion",
        "document contains duplicate ids",
        "drawing content changed before insertion",
        "edit identity collision",
        "group requires consecutive siblings to preserve stacking",
        "group requires objects in the same layer or parent",
        "invalid edit identity",
        "invalid selection",
        "invalid style edit",
        "invalid style value",
        "mixed formatting or multiple text runs; select simple single-run text",
        "non-finite transform",
        "replacement must be a single line of text",
        "select drawable objects",
        "select exactly one text object",
        "select objects inside the layer, not the layer itself",
        "selected objects are referenced; detach references before deleting",
        "selection changed before edit",
        "selection contains locked objects or belongs to a locked layer",
        "structural edits with stylesheets require inline styles first",
        "text paths and flowed text are unsupported",
        "ungroup supports plain groups without inherited style or effects",
        "unsupported edit operation",
    }
)
