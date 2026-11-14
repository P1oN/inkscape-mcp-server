// Manager update presentation. Verification, file mutation and restart stay in Rust.
- (void)bindManagerContext {
    if(self.managerContextRoot && ![self.managerContextRoot isEqual:self.installation]){self.managerOutcome=nil;self.managerWarning=nil;self.managerFailure=nil;self.startupFailure=nil;self.managerReview=nil;self.reviewOffer=nil;[self showManagerPage:nil];}
    self.managerContextRoot=[self.installation copy];
}
- (void)startupManagerFailed:(NSString *)message {self.managerFailureBeforeActivation=NO;self.startupFailure=message;[self inspect];}
- (void)startupInspectionFailed:(NSString *)message {NSString *failure=self.startupFailure;self.startupFailure=nil;[self failManagerUpdate:[NSString stringWithFormat:@"%@\nInstallation inspection also failed: %@",failure,message]];}
- (NSString *)managerVersion {return [[NSBundle mainBundle] objectForInfoDictionaryKey:@"InkscapeMCPDistribution"] ?: @"development";}
- (NSString *)displayValue:(id)value {return [value isKindOfClass:NSString.class] ? value : [value isKindOfClass:NSNumber.class] ? [value stringValue] : @"Not recorded";}
- (NSString *)shortIdentity:(id)value {NSString *text=[self displayValue:value];return text.length>16 ? [text substringToIndex:16] : text;}
- (NSButton *)pageButton:(NSString *)title action:(SEL)action {
    NSButton *button=[NSButton buttonWithTitle:title target:self action:action];button.bezelStyle=NSBezelStyleRounded;
    [button setContentCompressionResistancePriority:NSLayoutPriorityRequired forOrientation:NSLayoutConstraintOrientationHorizontal];return button;
}
- (void)stylePrimary:(NSButton *)button {
    button.bezelColor=[NSColor colorWithSRGBRed:0.41 green:0.34 blue:0.86 alpha:1];
    button.attributedTitle=[[NSAttributedString alloc] initWithString:button.title attributes:@{NSForegroundColorAttributeName:NSColor.whiteColor,NSFontAttributeName:[NSFont systemFontOfSize:13]}];
}
- (void)refreshManagerControls {
    BOOL idle=!self.running;
    self.pageSecondary.enabled=idle;
    self.pageManagerAction.enabled=idle && !self.preparation && self.managerCheckTask==nil;
    self.pagePrimary.enabled=idle && ![@[@"preparing",@"handoff"] containsObject:self.managerPage ?: @""];
    if([self.managerPage isEqual:@"settings"])self.pageSecondary.enabled=idle && self.hasInstallation && self.rollbackAvailable && !self.recoveryRequired && !self.preparation;
}
- (void)showManagerPage:(NSString *)page {self.managerPage=page;[self renderManagerPage];}
- (void)showSettings:(id)sender {if(!self.running && !self.managerReview)[self showManagerPage:@"settings"];}
- (void)backToConnection:(id)sender {if(!self.running && !self.managerReview)[self showManagerPage:nil];}
- (void)backToSettings:(id)sender {
    if(self.running)return;
    self.managerReview=nil;self.managerFailure=nil;self.managerFailureBeforeActivation=NO;[self showManagerPage:@"settings"];
}
- (void)retryManagerInspection:(id)sender {if(self.running)return;[self backToSettings:sender];[self inspect];}
- (void)settingsPrimary:(id)sender {if(self.managerOffer)[self updateManager:sender];else [self startUpdateCheck];}
- (void)rollbackFromSettings:(id)sender {if(self.running || !self.rollbackAvailable || self.recoveryRequired || self.preparation)return;[self backToConnection:sender];[self rollback:sender];}
- (void)presentManagerReview:(NSDictionary *)result {
    self.managerReview=result;[self showManagerPage:@"review"];
}
- (void)cancelManagerReview:(id)sender {
    if(self.running || !self.managerReview || self.detailsWindow)return;
    self.managerReview=nil;
    // Context changes invalidate Review before this action can run. A new preparation
    // still rechecks metadata, destination and bytes; cancellation never activates it.
    self.managerOffer=self.reviewOffer;self.reviewOffer=nil;
    self.managerAvailable.stringValue=self.managerOffer ? [NSString stringWithFormat:@"Manager %@ is available. Review the verified download before restarting.",self.managerOffer[@"tag"]] : @"Check Manager updates to review another candidate.";
    [self setPhase:@"ready" message:@"Manager update cancelled. No installed files changed."];[self showManagerPage:@"settings"];
}
- (void)confirmManagerReview:(id)sender {
    if(self.running || !self.managerReview || self.detailsWindow)return;
    NSDictionary *review=self.managerReview;self.managerReview=nil;self.reviewOffer=nil;
    self.managerFailureBeforeActivation=NO;
    [self beginManagerActivation:review];
}
- (void)failManagerUpdate:(NSString *)message {
    self.managerReview=nil;self.reviewOffer=nil;self.managerFailure=message;
    [self setPhase:@"failed" message:message];[self showManagerPage:@"error"];
}
- (NSView *)updateParagraph:(NSString *)text {
    NSTextField *label=managerLabel(text,13,NSColor.secondaryLabelColor);label.selectable=YES;label.maximumNumberOfLines=0;label.lineBreakMode=NSLineBreakByWordWrapping;return label;
}
- (NSView *)updateNotice:(NSString *)title detail:(NSString *)detail warning:(BOOL)warning {
    NSTextField *heading=managerLabel(title,14,warning ? NSColor.systemOrangeColor : NSColor.controlAccentColor);heading.font=[NSFont systemFontOfSize:14 weight:NSFontWeightSemibold];
    NSView *text=[self updateParagraph:detail];NSStackView *body=managerStack(@[heading,text],NSUserInterfaceLayoutOrientationVertical,10);managerFill(text,body);return managerCard(body);
}
- (NSView *)identityTable:(NSArray<NSArray<NSString *> *> *)rows {
    NSMutableArray *gridRows=[NSMutableArray new];
    for(NSArray *row in rows){NSMutableArray *cells=[NSMutableArray new];for(NSString *value in row){NSTextField *label=managerLabel(value,13,NSColor.labelColor);label.selectable=YES;label.maximumNumberOfLines=0;label.toolTip=value;[cells addObject:label];}[gridRows addObject:cells];}
    NSGridView *grid=[NSGridView gridViewWithViews:gridRows];grid.rowSpacing=30;grid.columnSpacing=20;grid.xPlacement=NSGridCellPlacementFill;grid.yPlacement=NSGridCellPlacementTop;
    [grid columnAtIndex:0].width=160;if(rows.firstObject.count==3)[grid columnAtIndex:2].width=90;
    return managerCard(grid);
}
- (void)addUpdateView:(NSView *)view to:(NSStackView *)stack {[stack addArrangedSubview:view];managerFill(view,stack);}
- (NSString *)reviewDetails {
    NSDictionary *review=self.managerReview,*target=review[@"target_manager"],*before=review[@"installed_bootstrap"];
    NSMutableString *text=[NSMutableString stringWithFormat:@"App: %@\nMCP root: %@\n\nManager application\nDistribution: %@ → %@\nBuild: %@ → %@\nSource: %@ → %@\nPublisher: %@\n\nInstalled bootstrap: %@\nTarget bootstrap: %@\n\n",review[@"manager_application"],self.installation,[self displayValue:review[@"installed_manager"][@"InkscapeMCPDistribution"]],target[@"tag"],review[@"installed_manager"][@"InkscapeMCPBuildID"],target[@"manager_build"],[self displayValue:review[@"installed_manager"][@"InkscapeMCPRevision"]],target[@"manager_revision"],target[@"publisher"],[self displayValue:[before isKindOfClass:NSDictionary.class] ? before[@"build_id"] : nil],target[@"bootstrap"][@"build_id"]];
    for(NSDictionary *change in review[@"bootstrap_changes"]){NSDictionary *from=change[@"from"];[text appendFormat:@"%@\nBefore SHA-256: %@\nAfter SHA-256: %@\n\n",change[@"destination"],[from isKindOfClass:NSDictionary.class] ? from[@"sha256"] : @"Absent",change[@"to"][@"sha256"]];}
    if(![review[@"bootstrap_changes"] count])[text appendString:@"Launcher files are unchanged. A fresh MCP root does not create a launcher just for a Manager update.\n\n"];
    if([before isKindOfClass:NSDictionary.class])[text appendFormat:@"Installed bootstrap source: %@\nInstalled bootstrap executable SHA-256: %@\n",[self displayValue:before[@"source_revision"]],[self displayValue:before[@"executable"][@"sha256"]]];
    [text appendFormat:@"Target bootstrap source: %@\nTarget bootstrap executable SHA-256: %@\nLauncher compatibility: %@\nText interface: %@\nHelper protocol: %@\nComponent/settings format: %@\nChannel: %@\n\nAny changed plan must be reviewed again.",target[@"bootstrap"][@"source_revision"],target[@"bootstrap"][@"executable"][@"sha256"],target[@"launcher_version"],target[@"text_interface"],target[@"helper_protocol"],target[@"component_format"],[target[@"prerelease"] boolValue] ? @"prerelease" : @"stable"];
    return text;
}
- (void)showExactChanges:(id)sender {
    if(!self.managerReview || self.detailsWindow)return;
    NSWindow *sheet=[[NSWindow alloc] initWithContentRect:NSMakeRect(0,0,760,716) styleMask:NSWindowStyleMaskTitled|NSWindowStyleMaskResizable backing:NSBackingStoreBuffered defer:NO];sheet.title=@"Reviewed file changes";sheet.contentMinSize=NSMakeSize(600,420);self.detailsWindow=sheet;
    NSTextField *heading=managerLabel(@"Reviewed file changes",20,NSColor.labelColor);heading.font=[NSFont systemFontOfSize:20 weight:NSFontWeightSemibold];heading.translatesAutoresizingMaskIntoConstraints=NO;[sheet.contentView addSubview:heading];
    NSScrollView *scroll=[NSScrollView new];scroll.hasVerticalScroller=YES;scroll.autohidesScrollers=YES;scroll.drawsBackground=NO;scroll.translatesAutoresizingMaskIntoConstraints=NO;[sheet.contentView addSubview:scroll];
    NSTextView *text=[NSTextView new];text.editable=NO;text.selectable=YES;text.font=[NSFont monospacedSystemFontOfSize:12 weight:NSFontWeightRegular];text.string=[self reviewDetails];text.textContainerInset=NSMakeSize(12,12);text.autoresizingMask=NSViewWidthSizable;text.textContainer.widthTracksTextView=YES;scroll.documentView=text;
    NSButton *back=[self pageButton:@"Back to review" action:@selector(closeExactChanges:)];back.translatesAutoresizingMaskIntoConstraints=NO;[sheet.contentView addSubview:back];
    [NSLayoutConstraint activateConstraints:@[[heading.leadingAnchor constraintEqualToAnchor:sheet.contentView.leadingAnchor constant:28],[heading.topAnchor constraintEqualToAnchor:sheet.contentView.topAnchor constant:24],[scroll.leadingAnchor constraintEqualToAnchor:sheet.contentView.leadingAnchor constant:20],[scroll.trailingAnchor constraintEqualToAnchor:sheet.contentView.trailingAnchor constant:-20],[scroll.topAnchor constraintEqualToAnchor:heading.bottomAnchor constant:16],[scroll.bottomAnchor constraintEqualToAnchor:back.topAnchor constant:-16],[back.trailingAnchor constraintEqualToAnchor:sheet.contentView.trailingAnchor constant:-24],[back.bottomAnchor constraintEqualToAnchor:sheet.contentView.bottomAnchor constant:-20]]];
    [self.window beginSheet:sheet completionHandler:nil];
}
- (void)closeExactChanges:(id)sender {[self.window endSheet:self.detailsWindow];self.detailsWindow=nil;}
- (void)renderManagerPage {
    BOOL visible=self.managerPage!=nil;self.managerPanel.hidden=!visible;self.connectionScroll.hidden=visible;self.connectionStatus.hidden=visible;self.connectionActions.hidden=visible;
    for(NSView *view in self.managerPanel.subviews.copy)[view removeFromSuperview];self.pagePrimary=nil;self.pageSecondary=nil;self.pageManagerAction=nil;self.pageProgress=nil;
    if(!visible){self.summary.stringValue=self.hasInstallation ? [NSString stringWithFormat:@"Runtime installed · %@",self.installedComponents[@"distribution"] ?: @"Current runtime"] : @"Connect your SVG workspace to your AI tools.";return;}
    NSString *page=self.managerPage;BOOL settings=[page isEqual:@"settings"],review=[page isEqual:@"review"],preparing=[page isEqual:@"preparing"],handoff=[page isEqual:@"handoff"],recovery=[page isEqual:@"recovery"];
    self.summary.stringValue=settings ? @"Application, runtime and instructions have separate identities" : review ? @"Review the application and launcher update together" : preparing ? @"Preparing a candidate; installed files are unchanged" : handoff ? @"The reviewed update is being handed off" : recovery ? @"An interrupted Manager update needs attention" : @"Update stopped before completion";
    NSString *heading=settings ? @"Settings & versions" : review ? [NSString stringWithFormat:@"Update Manager to %@?",self.managerReview[@"target_manager"][@"tag"]] : preparing ? @"Preparing the Manager update" : handoff ? @"Restarting Manager to finish" : recovery ? @"The Manager update needs recovery" : @"Manager cannot update this copy";
    NSString *subtitle=settings ? @"Review installed components and update Manager separately." : review ? @"Only Manager will restart. Existing MCP sessions keep running." : preparing ? @"Download and verification must finish before you can confirm." : handoff ? @"This window will close briefly. The updated Manager will reopen." : recovery ? @"The updater could not confirm the new Manager starting successfully." : @"Review the failure below before trying again.";
    NSStackView *body=managerStack(@[],NSUserInterfaceLayoutOrientationVertical,20);body.translatesAutoresizingMaskIntoConstraints=NO;
    NSTextField *title=managerLabel(heading,24,NSColor.labelColor);title.font=[NSFont systemFontOfSize:24 weight:NSFontWeightSemibold];[self addUpdateView:title to:body];[self addUpdateView:[self updateParagraph:subtitle] to:body];
    NSString *footerTitle=nil,*footerDetail=nil,*primary=nil;SEL action=NULL;
    if(settings){
        NSString *banner=[self.managerOutcome isEqual:@"restored"] ? @"Previous Manager restored" : [self.managerOutcome isEqual:@"updated"] ? [NSString stringWithFormat:@"Manager %@ is installed",[self managerVersion]] : self.managerOffer ? [NSString stringWithFormat:@"Manager %@ is available",self.managerOffer[@"tag"]] : [NSString stringWithFormat:@"Manager %@",[self managerVersion]];
        NSString *detail=[self.managerOutcome isEqual:@"restored"] ? @"The previous installed Manager and launcher were restored." : [self.managerOutcome isEqual:@"updated"] ? @"Manager reopened successfully. MCP sessions were left running." : self.managerOffer ? [NSString stringWithFormat:@"Installed: %@ · restarts Manager only",[self managerVersion]] : self.managerAvailable.stringValue;
        if(self.managerCheckTask && !self.managerOutcome){banner=@"Checking Manager updates…";detail=@"The check only reads metadata. Local setup remains available.";}
        self.pageManagerAction=[self pageButton:self.managerOffer ? @"Review Manager update" : @"Check Manager updates" action:@selector(settingsPrimary:)];
        NSTextField *bannerTitle=managerLabel(banner,14,NSColor.labelColor);bannerTitle.font=[NSFont systemFontOfSize:14 weight:NSFontWeightSemibold];
        [self stylePrimary:self.pageManagerAction];
        NSView *bannerDetail=[self updateParagraph:detail];NSStackView *bannerText=managerStack(@[bannerTitle,bannerDetail],NSUserInterfaceLayoutOrientationVertical,10);managerFill(bannerDetail,bannerText);
        NSStackView *bannerRow=managerStack(@[bannerText,managerSpacer(),self.pageManagerAction],NSUserInterfaceLayoutOrientationHorizontal,16);
        [self addUpdateView:managerCard(bannerRow) to:body];
        NSString *emptyIdentity=[self.state isEqual:@"not_installed"] ? @"Not installed" : @"Not verified";
        NSDictionary *v=self.installedComponents;NSString *managerBuild=[[NSBundle mainBundle] objectForInfoDictionaryKey:@"InkscapeMCPBuildID"];
        [self addUpdateView:[self identityTable:@[@[@"Component",@"Installed identity",@"Update scope"],@[@"Running Manager",[NSString stringWithFormat:@"%@ · build %@",[self managerVersion],[self shortIdentity:managerBuild]],@"Application"],@[@"Runtime",v ? [NSString stringWithFormat:@"Build %@ · from %@",[self shortIdentity:v[@"runtime_build"]],[self displayValue:v[@"runtime_distribution"] ?: v[@"distribution"]]] : emptyIdentity,@"Components"],@[@"Agent instructions & skill",v ? [NSString stringWithFormat:@"Content %@ · version %@",[self shortIdentity:v[@"instructions_content"] ?: v[@"instructions_version"]],[self displayValue:v[@"instructions_version"]]] : emptyIdentity,@"Components"],@[@"Launcher",v ? [NSString stringWithFormat:@"Build %@ · compatibility %@",[self shortIdentity:v[@"launcher_build"]],[self displayValue:v[@"launcher_version"]]] : emptyIdentity,@"If changed"]]] to:body];
        NSView *location=[self updateParagraph:[NSString stringWithFormat:@"%@ · new monitoring off\nRunning Manager: %@",self.channel.titleOfSelectedItem,[NSBundle mainBundle].bundlePath]];
        NSButton *technical=[self pageButton:@"Technical details" action:@selector(showTechnicalDetails:)];
        [self addUpdateView:managerCard(managerStack(@[location,managerSpacer(),technical],NSUserInterfaceLayoutOrientationHorizontal,16)) to:body];
        if(self.managerWarning)[self addUpdateView:[self updateParagraph:self.managerWarning] to:body];
        footerTitle=[self.managerOutcome isEqual:@"restored"] ? @"Manager recovery complete" : [self.managerOutcome isEqual:@"updated"] ? @"Manager update complete" : @"Installed versions are independent";
        footerDetail=[self.managerOutcome isEqual:@"restored"] ? @"Runtime, instructions and existing MCP sessions were preserved." : [self.managerOutcome isEqual:@"updated"] ? @"No Codex reconnect is required for this Manager-only change." : @"Component rollback affects runtime and instructions only.";
        primary=@"Back to connection";action=@selector(backToConnection:);
    }else if(review){
        NSDictionary *r=self.managerReview,*target=r[@"target_manager"],*old=r[@"installed_manager"],*bootstrap=r[@"installed_bootstrap"];
NSString *launcherChange=[r[@"bootstrap_changes"] count] ? [NSString stringWithFormat:@"%@ → %@ · compatibility %@",[self shortIdentity:[bootstrap isKindOfClass:NSDictionary.class] ? bootstrap[@"build_id"] : nil],[self shortIdentity:target[@"bootstrap"][@"build_id"]],target[@"launcher_version"]] : [bootstrap isKindOfClass:NSDictionary.class] ? [NSString stringWithFormat:@"Kept · %@",[self shortIdentity:bootstrap[@"build_id"]]] : @"Not installed; no launcher will be created";
        [self addUpdateView:[self updateNotice:@"Candidate and publisher verified" detail:[NSString stringWithFormat:@"Developer ID matches the expected publisher · %@",target[@"publisher"]] warning:NO] to:body];
        [self addUpdateView:[self identityTable:@[@[@"Manager",[NSString stringWithFormat:@"%@ → %@",[self displayValue:old[@"InkscapeMCPDistribution"]],target[@"tag"]]],@[@"Manager build",[NSString stringWithFormat:@"%@ → %@",[self shortIdentity:old[@"InkscapeMCPBuildID"]],[self shortIdentity:target[@"manager_build"]]]],@[@"Launcher build",launcherChange],@[@"Destination",r[@"manager_application"]],@[@"Runtime & instructions",@"Keep current selected components"]]] to:body];
        [self addUpdateView:[self updateParagraph:@"Workspaces, complete skills, client settings and monitoring preferences are kept. Only the reviewed Manager and launcher files will change."] to:body];
        if(![r[@"baseline_healthy"] boolValue])[self addUpdateView:[self updateNotice:@"Existing MCP installation needs attention" detail:@"Its baseline is unverified or incomplete. This updates Manager while retaining that state; review the installation separately if needed." warning:YES] to:body];
        NSButton *exact=[self pageButton:@"View exact file changes" action:@selector(showExactChanges:)];
        [self addUpdateView:managerStack(@[exact,managerSpacer()],NSUserInterfaceLayoutOrientationHorizontal,0) to:body];
        footerTitle=@"Restart Manager after confirmation";footerDetail=@"Codex, Claude and Inkscape are not restarted.";primary=@"Update Manager and restart";action=@selector(confirmManagerReview:);
    }else if(preparing){
        [self addUpdateView:managerCard([self updateParagraph:@"1   Read the selected release metadata\n\n2   Download the Manager candidate\n\n3   Verify archive, application and publisher\n\n4   Prepare the exact app and launcher change plan"]) to:body];
        [self addUpdateView:[self updateParagraph:@"No app has been replaced. Manager will show the verified plan next.\nClosing is protected while the preparation operation is running."] to:body];
        self.pageProgress=(NSTextField *)[self updateParagraph:@"Preparing and verifying the candidate…"];[self addUpdateView:self.pageProgress to:body];footerTitle=@"Preparing the review";footerDetail=@"Runtime and instructions stay selected.";primary=@"Preparing…";
    }else if(handoff){
        [self addUpdateView:[self updateNotice:@"The reviewed update is being handed off" detail:@"This Manager exits before its installed application is replaced." warning:NO] to:body];
        [self addUpdateView:managerCard([self updateParagraph:@"• Existing MCP sessions keep their current runtime paths\n• Codex and Claude are not restarted\n• Inkscape windows stay open\n• Workspace, instructions, skills and client settings are kept\n• The same Manager profile and installation reopen"]) to:body];
        footerTitle=@"Manager restart in progress";footerDetail=@"Do not start a second copy during replacement.";primary=@"Restarting…";
    }else if(recovery){
        [self addUpdateView:[self updateNotice:@"Review the app state before recovery" detail:@"The retained Manager and launcher recovery information is available." warning:YES] to:body];
        [self addUpdateView:managerCard([self updateParagraph:@"Recovery restores the previous Manager and launcher when the destination is idle.\n\nIf another Manager window you opened uses this app, close that window first. Manager will not close other applications for you.\n\nThe selected runtime, instructions, skills and workspace are kept. If file identities no longer match the saved plan, recovery stops with details."]) to:body];
        if(self.managerFailure)[self addUpdateView:[self updateParagraph:self.managerFailure] to:body];
        footerTitle=@"Recover the app and launcher";footerDetail=@"Recovery rechecks destinations before making changes.";primary=@"Recover Manager";action=@selector(recover:);
    }else {
        [self addUpdateView:[self updateNotice:self.managerFailureBeforeActivation ? @"No installed files were changed" : @"Update was not completed" detail:self.managerFailure ?: @"Return to settings and review again." warning:YES] to:body];
        [self addUpdateView:managerCard([self updateParagraph:@"Manager will not change system permissions or install a second copy elsewhere.\n\nIf this app location is not writable, quit Manager and ask an administrator to update this installed copy, or restore write access through your normal application management process.\n\nYou can keep using the current runtime and instructions. If activation was interrupted, recovery information is retained. Choose Retry inspection to check it, or return to settings to choose another installation."]) to:body];
        footerTitle=self.managerFailureBeforeActivation ? @"Update was not applied" : @"Update was not confirmed";footerDetail=@"Resolve the reported problem before reviewing another update.";primary=@"Back to settings";action=@selector(backToSettings:);
    }
    NSScrollView *scroll=[NSScrollView new];scroll.drawsBackground=NO;scroll.hasVerticalScroller=YES;scroll.autohidesScrollers=YES;scroll.translatesAutoresizingMaskIntoConstraints=NO;[self.managerPanel addSubview:scroll];
    NSView *document=[ManagerDocumentView new];document.translatesAutoresizingMaskIntoConstraints=NO;scroll.documentView=document;[document addSubview:body];
    [NSLayoutConstraint activateConstraints:@[[document.widthAnchor constraintEqualToAnchor:scroll.contentView.widthAnchor],[body.topAnchor constraintEqualToAnchor:document.topAnchor],[body.leadingAnchor constraintEqualToAnchor:document.leadingAnchor],[body.trailingAnchor constraintEqualToAnchor:document.trailingAnchor],[body.bottomAnchor constraintEqualToAnchor:document.bottomAnchor]]];
    NSTextField *footerHeading=managerLabel(footerTitle,13,NSColor.labelColor);NSView *footerText=[self updateParagraph:footerDetail];NSStackView *footerCopy=managerStack(@[footerHeading,footerText],NSUserInterfaceLayoutOrientationVertical,10);managerFill(footerText,footerCopy);
    self.pagePrimary=[self pageButton:primary action:action];self.pagePrimary.keyEquivalent=review ? @"\r" : @"";[self stylePrimary:self.pagePrimary];
    NSMutableArray *controls=[NSMutableArray arrayWithObject:self.pagePrimary];
    if(settings || review){self.pageSecondary=[self pageButton:review ? @"Cancel" : @"Rollback" action:review ? @selector(cancelManagerReview:) : @selector(rollbackFromSettings:)];[controls insertObject:self.pageSecondary atIndex:0];}
    else if([page isEqual:@"error"]){self.pageSecondary=[self pageButton:@"Retry inspection" action:@selector(retryManagerInspection:)];[controls insertObject:self.pageSecondary atIndex:0];}
    NSStackView *actions=managerStack(controls,NSUserInterfaceLayoutOrientationHorizontal,12);
    NSStackView *footer=managerStack(@[footerCopy,managerSpacer(),actions],NSUserInterfaceLayoutOrientationHorizontal,16);footer.translatesAutoresizingMaskIntoConstraints=NO;[self.managerPanel addSubview:footer];
    [NSLayoutConstraint activateConstraints:@[[scroll.topAnchor constraintEqualToAnchor:self.managerPanel.topAnchor],[scroll.leadingAnchor constraintEqualToAnchor:self.managerPanel.leadingAnchor],[scroll.trailingAnchor constraintEqualToAnchor:self.managerPanel.trailingAnchor],[scroll.bottomAnchor constraintEqualToAnchor:footer.topAnchor constant:-24],[footer.leadingAnchor constraintEqualToAnchor:self.managerPanel.leadingAnchor],[footer.trailingAnchor constraintEqualToAnchor:self.managerPanel.trailingAnchor],[footer.bottomAnchor constraintEqualToAnchor:self.managerPanel.bottomAnchor]]];
    [self refreshControls];
}
- (void)showTechnicalDetails:(id)sender {
    [self backToConnection:sender];self.advancedButton.state=NSControlStateValueOn;[self toggleAdvanced:nil];
}
