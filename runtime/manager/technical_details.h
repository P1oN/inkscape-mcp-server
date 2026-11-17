// Compact presentation only. Full identities remain bound to their original values.
- (NSString *)componentIdentityState {
    return [self.state isEqual:@"not_installed"] ? @"Not installed" : @"Not verified";
}
- (NSString *)runtimeOrigin {
    id origin=self.installedComponents[@"runtime_distribution"];
    if(![origin isKindOfClass:NSString.class] || ![origin length])return @"Installed";
    return [origin isEqual:@"bundled"] ? @"Bundled" : [NSString stringWithFormat:@"From %@",origin];
}
- (NSView *)componentCell:(NSString *)title detail:(NSString *)detail monospaced:(BOOL)monospaced {
    NSTextField *primary=managerLabel(title,13,NSColor.labelColor);primary.font=[NSFont systemFontOfSize:13 weight:NSFontWeightMedium];
    primary.maximumNumberOfLines=1;primary.lineBreakMode=NSLineBreakByTruncatingMiddle;primary.selectable=YES;primary.toolTip=title;
    NSTextField *secondary=managerLabel(detail,11,NSColor.secondaryLabelColor);secondary.selectable=YES;
    secondary.maximumNumberOfLines=1;secondary.lineBreakMode=NSLineBreakByTruncatingMiddle;
    if(monospaced)secondary.font=[NSFont monospacedSystemFontOfSize:11 weight:NSFontWeightRegular];
    NSStackView *cell=managerStack(@[primary,secondary],NSUserInterfaceLayoutOrientationVertical,4);
    managerFill(primary,cell);managerFill(secondary,cell);return cell;
}
- (NSView *)installedComponentSummary {
    NSDictionary *v=self.installedComponents;NSString *empty=[self componentIdentityState];
    NSString *build=[[NSBundle mainBundle] objectForInfoDictionaryKey:@"InkscapeMCPBuildID"];
    NSArray *rows=@[
        @[@"Running Manager",@"Desktop application",[self managerVersion],[NSString stringWithFormat:@"Build %@",[self shortIdentity:build]],@"Application"],
        @[@"Runtime",@"MCP server",v ? [self runtimeOrigin] : empty,v ? [NSString stringWithFormat:@"Build %@",[self shortIdentity:v[@"runtime_build"]]] : @"",@"Components"],
        @[@"Agent instructions & skill",@"Agent guidance",v ? @"Installed" : empty,v ? [NSString stringWithFormat:@"Content %@",[self shortIdentity:v[@"instructions_content"] ?: v[@"instructions_version"]]] : @"",@"Components"],
        @[@"Launcher",@"Permanent entry point",v ? [NSString stringWithFormat:@"Compatibility %@",[self displayValue:v[@"launcher_version"]]] : empty,v ? [NSString stringWithFormat:@"Build %@",[self shortIdentity:v[@"launcher_build"]]] : @"",@"When required"]
    ];
    NSMutableArray *gridRows=[NSMutableArray new];NSMutableArray *headings=[NSMutableArray new];
    for(NSString *heading in @[@"Component",@"Installed",@"Updates with"])[headings addObject:managerLabel(heading,12,NSColor.secondaryLabelColor)];
    [gridRows addObject:headings];
    for(NSArray *row in rows){
        NSTextField *scope=managerLabel(row[4],12,NSColor.secondaryLabelColor);
        [gridRows addObject:@[[self componentCell:row[0] detail:row[1] monospaced:NO],[self componentCell:row[2] detail:row[3] monospaced:YES],scope]];
    }
    NSGridView *grid=[NSGridView gridViewWithViews:gridRows];grid.identifier=@"component-summary";
    grid.rowSpacing=20;grid.columnSpacing=20;grid.xPlacement=NSGridCellPlacementFill;grid.yPlacement=NSGridCellPlacementCenter;
    [grid columnAtIndex:0].width=264;[grid columnAtIndex:2].width=100;
    return managerCard(grid);
}
- (NSView *)compactManagerLocation {
    NSString *path=[NSBundle mainBundle].bundlePath;
    NSTextField *channel=managerLabel([NSString stringWithFormat:@"%@ · New monitoring off",self.channel.titleOfSelectedItem],12,NSColor.secondaryLabelColor);
    channel.maximumNumberOfLines=1;channel.lineBreakMode=NSLineBreakByTruncatingMiddle;
    NSTextField *location=[self pathLabel:[NSString stringWithFormat:@"App: …/%@",path.lastPathComponent]];
    location.identifier=@"compact-manager-path";location.maximumNumberOfLines=1;location.toolTip=path;location.accessibilityLabel=[NSString stringWithFormat:@"Running Manager: %@",path];
    NSStackView *text=managerStack(@[channel,location],NSUserInterfaceLayoutOrientationVertical,6);managerFill(channel,text);managerFill(location,text);
    [text setContentHuggingPriority:1 forOrientation:NSLayoutConstraintOrientationHorizontal];
    NSButton *details=[self pageButton:@"Build IDs & paths" action:@selector(showTechnicalDetails:)];
    return managerCard(managerStack(@[text,managerSpacer(),details],NSUserInterfaceLayoutOrientationHorizontal,16));
}
- (NSArray<NSDictionary<NSString *,NSString *> *> *)technicalFields {
    NSDictionary *v=self.installedComponents;NSBundle *bundle=NSBundle.mainBundle;
    NSString *manager=[NSString stringWithFormat:@"Build %@ · %@\nSource %@",[self displayValue:[bundle objectForInfoDictionaryKey:@"InkscapeMCPBuildID"]],[self managerVersion],[self displayValue:[bundle objectForInfoDictionaryKey:@"InkscapeMCPRevision"]]];
    NSString *runtime=v ? [NSString stringWithFormat:@"Build %@ · %@\nSource %@",[self displayValue:v[@"runtime_build"]],[self runtimeOrigin],[self displayValue:v[@"runtime_revision"]]] : [self componentIdentityState];
    NSString *instructions=v ? [NSString stringWithFormat:@"Content %@\nVersion %@",[self displayValue:v[@"instructions_content"]],[self displayValue:v[@"instructions_version"]]] : [self componentIdentityState];
    NSString *launcher=v ? [NSString stringWithFormat:@"Build %@ · compatibility %@\nSource %@",[self displayValue:v[@"launcher_build"]],[self displayValue:v[@"launcher_version"]],[self displayValue:v[@"launcher_revision"]]] : [self componentIdentityState];
    NSDictionary *signature=[v[@"manager_helper_signing"] isKindOfClass:NSDictionary.class] ? v[@"manager_helper_signing"] : nil;
    NSString *signing=[signature[@"status"] isEqual:@"developer_id_verified"] ? [NSString stringWithFormat:@"Developer ID verified (%@)",[self displayValue:signature[@"team_id"]]] : [signature[@"status"] isEqual:@"ad_hoc"] ? @"Development (ad-hoc)" : @"Not verified";
    return @[
        @{@"label":@"Running Manager",@"value":manager},
        @{@"label":@"Runtime",@"value":runtime},
        @{@"label":@"Agent instructions & skill",@"value":instructions},
        @{@"label":@"Launcher",@"value":launcher},
        @{@"label":@"Running Manager · full path",@"value":bundle.bundlePath},
        @{@"label":@"MCP installation · full path",@"value":[self displayValue:self.installation]},
        @{@"label":@"Inkscape · executable path",@"value":self.inkscape ?: @"Not found"},
        @{@"label":@"Bundled helper signing",@"value":signing},
        @{@"label":@"Notarization on this Mac",@"value":@"Not checked"}
    ];
}
- (NSPasteboard *)technicalPasteboard {return NSPasteboard.generalPasteboard;}
- (void)copyTechnicalValue:(ManagerCopyButton *)button {
    NSPasteboard *pasteboard=[self technicalPasteboard];[pasteboard clearContents];
    BOOL copied=[pasteboard setString:button.clipboardValue forType:NSPasteboardTypeString];
    button.title=copied ? @"Copied ✓" : @"Copy failed";NSUInteger generation=++button.feedbackGeneration;
    NSAccessibilityPostNotificationWithUserInfo(NSApp,NSAccessibilityAnnouncementRequestedNotification,@{NSAccessibilityAnnouncementKey:copied ? @"Copied" : @"Copy failed",NSAccessibilityPriorityKey:@(NSAccessibilityPriorityMedium)});
    __weak ManagerCopyButton *weakButton=button;
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW,2*NSEC_PER_SEC),dispatch_get_main_queue(),^{
        ManagerCopyButton *current=weakButton;if(current && current.feedbackGeneration==generation)current.title=current.normalTitle;
    });
}
- (ManagerCopyButton *)technicalCopyButton:(NSString *)title value:(NSString *)value label:(NSString *)label {
    ManagerCopyButton *button=[ManagerCopyButton buttonWithTitle:title target:self action:@selector(copyTechnicalValue:)];
    button.bezelStyle=NSBezelStyleRounded;button.clipboardValue=value;button.normalTitle=title;
    button.accessibilityLabel=[NSString stringWithFormat:@"%@ — %@",title,label];
    [button.widthAnchor constraintGreaterThanOrEqualToConstant:MAX(96,button.intrinsicContentSize.width)].active=YES;
    [button setContentCompressionResistancePriority:NSLayoutPriorityRequired forOrientation:NSLayoutConstraintOrientationHorizontal];return button;
}
- (void)showTechnicalDetails:(id)sender {
    if(self.running || self.managerReview || self.detailsWindow)return;
    NSArray *fields=[self technicalFields];
    NSWindow *sheet=[[NSWindow alloc] initWithContentRect:NSMakeRect(0,0,700,744) styleMask:NSWindowStyleMaskTitled|NSWindowStyleMaskResizable backing:NSBackingStoreBuffered defer:NO];
    sheet.title=@"Build IDs & paths";sheet.contentMinSize=NSMakeSize(480,400);self.detailsWindow=sheet;
    NSTextField *heading=managerLabel(sheet.title,22,NSColor.labelColor);heading.font=[NSFont systemFontOfSize:22 weight:NSFontWeightSemibold];
    NSTextField *guidance=managerLabel(@"Full values are shown below. Copy keeps the complete value.",12,NSColor.secondaryLabelColor);
    NSStackView *header=managerStack(@[heading,guidance],NSUserInterfaceLayoutOrientationVertical,8);managerFill(guidance,header);
    header.translatesAutoresizingMaskIntoConstraints=NO;[sheet.contentView addSubview:header];
    NSScrollView *scroll=[NSScrollView new];scroll.drawsBackground=NO;scroll.hasVerticalScroller=YES;scroll.autohidesScrollers=YES;scroll.translatesAutoresizingMaskIntoConstraints=NO;[sheet.contentView addSubview:scroll];
    NSView *document=[ManagerDocumentView new];document.translatesAutoresizingMaskIntoConstraints=NO;scroll.documentView=document;
    NSStackView *body=managerStack(@[],NSUserInterfaceLayoutOrientationVertical,16);body.translatesAutoresizingMaskIntoConstraints=NO;[document addSubview:body];
    NSMutableArray *all=[NSMutableArray new];
    for(NSDictionary *field in fields){
        NSString *label=field[@"label"],*value=field[@"value"];[all addObject:[NSString stringWithFormat:@"%@\n%@",label,value]];
        NSTextField *name=managerLabel(label,12,NSColor.secondaryLabelColor);
        NSTextField *text=managerLabel(value,12,NSColor.labelColor);text.font=[NSFont monospacedSystemFontOfSize:12 weight:NSFontWeightRegular];
        text.selectable=YES;text.maximumNumberOfLines=0;text.lineBreakMode=value.isAbsolutePath ? NSLineBreakByWordWrapping : NSLineBreakByCharWrapping;text.identifier=@"technical-value";text.accessibilityLabel=label;
        [text setContentCompressionResistancePriority:250 forOrientation:NSLayoutConstraintOrientationHorizontal];
        ManagerCopyButton *copy=[self technicalCopyButton:@"Copy" value:value label:label];
        NSStackView *row=managerStack(@[text,managerSpacer(),copy],NSUserInterfaceLayoutOrientationHorizontal,16);row.alignment=NSLayoutAttributeTop;
        NSStackView *section=managerStack(@[name,row],NSUserInterfaceLayoutOrientationVertical,8);managerFill(row,section);
        [self addUpdateView:section to:body];
        NSBox *separator=[NSBox new];separator.boxType=NSBoxSeparator;[self addUpdateView:separator to:body];
    }
    ManagerCopyButton *copyAll=[self technicalCopyButton:@"Copy all details" value:[all componentsJoinedByString:@"\n\n"] label:@"Build IDs & paths"];
    NSButton *done=[self pageButton:@"Done" action:@selector(closeExactChanges:)];done.keyEquivalent=@"\033";[self stylePrimary:done];
    NSStackView *footer=managerStack(@[copyAll,managerSpacer(),done],NSUserInterfaceLayoutOrientationHorizontal,16);footer.translatesAutoresizingMaskIntoConstraints=NO;[sheet.contentView addSubview:footer];
    [NSLayoutConstraint activateConstraints:@[
        [header.leadingAnchor constraintEqualToAnchor:sheet.contentView.leadingAnchor constant:28],[header.trailingAnchor constraintEqualToAnchor:sheet.contentView.trailingAnchor constant:-28],[header.topAnchor constraintEqualToAnchor:sheet.contentView.topAnchor constant:24],
        [scroll.leadingAnchor constraintEqualToAnchor:sheet.contentView.leadingAnchor constant:28],[scroll.trailingAnchor constraintEqualToAnchor:sheet.contentView.trailingAnchor constant:-28],[scroll.topAnchor constraintEqualToAnchor:header.bottomAnchor constant:20],[scroll.bottomAnchor constraintEqualToAnchor:footer.topAnchor constant:-20],
        [document.widthAnchor constraintEqualToAnchor:scroll.contentView.widthAnchor],[body.leadingAnchor constraintEqualToAnchor:document.leadingAnchor],[body.trailingAnchor constraintEqualToAnchor:document.trailingAnchor],[body.topAnchor constraintEqualToAnchor:document.topAnchor],[body.bottomAnchor constraintEqualToAnchor:document.bottomAnchor],
        [footer.leadingAnchor constraintEqualToAnchor:sheet.contentView.leadingAnchor constant:28],[footer.trailingAnchor constraintEqualToAnchor:sheet.contentView.trailingAnchor constant:-28],[footer.bottomAnchor constraintEqualToAnchor:sheet.contentView.bottomAnchor constant:-24]
    ]];
    [self.window beginSheet:sheet completionHandler:nil];
}
