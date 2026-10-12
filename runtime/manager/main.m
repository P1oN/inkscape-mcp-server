#import <Cocoa/Cocoa.h>
#import "task_io.h"

static NSString *managerHome(void) {
    NSString *value=NSProcessInfo.processInfo.environment[@"HOME"];
    return value.isAbsolutePath ? value : NSHomeDirectory();
}

static NSTextField *managerLabel(NSString *text, CGFloat size, NSColor *color) {
    NSTextField *label=[NSTextField wrappingLabelWithString:text];label.font=[NSFont systemFontOfSize:size];label.textColor=color;return label;
}
static NSStackView *managerStack(NSArray<NSView *> *views, NSUserInterfaceLayoutOrientation orientation, CGFloat spacing) {
    NSStackView *stack=[NSStackView stackViewWithViews:views];stack.orientation=orientation;stack.alignment=orientation==NSUserInterfaceLayoutOrientationVertical ? NSLayoutAttributeLeading : NSLayoutAttributeCenterY;stack.spacing=spacing;return stack;
}
static NSView *managerSpacer(void) {
    NSView *view=[NSView new];[view setContentHuggingPriority:1 forOrientation:NSLayoutConstraintOrientationHorizontal];return view;
}
static NSBox *managerCard(NSView *content) {
    NSBox *box=[NSBox new];box.boxType=NSBoxCustom;box.borderColor=NSColor.separatorColor;box.borderWidth=0.5;box.cornerRadius=10;box.fillColor=NSColor.controlBackgroundColor;box.contentViewMargins=NSZeroSize;
    content.translatesAutoresizingMaskIntoConstraints=NO;[box.contentView addSubview:content];
    [NSLayoutConstraint activateConstraints:@[[content.leadingAnchor constraintEqualToAnchor:box.contentView.leadingAnchor constant:16],[content.trailingAnchor constraintEqualToAnchor:box.contentView.trailingAnchor constant:-16],[content.topAnchor constraintEqualToAnchor:box.contentView.topAnchor constant:14],[content.bottomAnchor constraintEqualToAnchor:box.contentView.bottomAnchor constant:-14]]];return box;
}
static void managerFill(NSView *view, NSView *parent) {[view.widthAnchor constraintEqualToAnchor:parent.widthAnchor].active=YES;}

@interface ManagerDocumentView : NSView
@end
@implementation ManagerDocumentView
- (BOOL)isFlipped {return YES;}
@end

// Thin native UI. Discovery, verification, preservation and recovery belong to Rust.
@interface Manager : NSObject <NSApplicationDelegate, NSWindowDelegate>
@property NSWindow *window;
@property NSTextField *installed;
@property NSTextField *installationLabel;
@property NSTextField *available;
@property NSTextField *status;
@property NSTextField *statusTitle;
@property NSTextField *summary;
@property NSTextField *clientHint;
@property NSTextField *channelHint;
@property NSImageView *statusIcon;
@property NSBox *advancedCard;
@property NSButton *advancedButton;
@property NSButton *checkButton;
@property NSButton *recoverButton;
@property NSButton *rollbackButton;
@property NSMutableArray<NSButton *> *updateButtons;
@property NSTextField *workspaceLabel;
@property NSTextField *inkscapeLabel;
@property NSPopUpButton *channel;
@property NSButton *previewChannelButton;
@property NSPopUpButton *candidates;
@property NSProgressIndicator *progress;
@property NSMutableArray<NSButton *> *buttons;
@property NSButton *installButton;
@property NSButton *cancelButton;
@property NSButton *codex;
@property NSButton *claude;
@property NSButton *skills;
@property NSString *installation;
@property NSString *workspace;
@property NSString *inkscape;
@property NSString *legacySource;
@property NSString *preparation;
@property NSString *state;
@property NSTask *running;
@property BOOL hasInstallation;
@property BOOL recoveryRequired;
@property BOOL rollbackAvailable;
@property BOOL updateAvailable;
@property BOOL configurationChanged;
@property NSArray<NSDictionary *> *clientDiscovery;
@end
@implementation Manager
- (NSButton *)button:(NSString *)title action:(SEL)action {
    NSButton *button=[NSButton buttonWithTitle:title target:self action:action];
    button.bezelStyle=NSBezelStyleRounded;
    [self.buttons addObject:button]; return button;
}
- (void)setPhase:(NSString *)phase message:(NSString *)message {
    self.state=phase; self.status.stringValue=message; self.status.toolTip=message;
    BOOL failed=[phase isEqualToString:@"failed"] || [phase isEqualToString:@"damaged"];
    BOOL busy=self.running!=nil || [phase isEqualToString:@"loading"];
    BOOL setup=!self.hasInstallation && ![phase isEqualToString:@"prepared"];
    self.statusTitle.stringValue=failed ? @"Needs attention" : busy ? @"Working…" : [phase isEqualToString:@"prepared"] ? @"Ready to install" : [phase isEqualToString:@"completed"] ? @"All set" : setup ? @"Finish setup" : @"Ready";
    self.statusIcon.image=[NSImage imageWithSystemSymbolName:failed ? @"exclamationmark.circle.fill" : busy ? @"arrow.triangle.2.circlepath" : setup ? @"info.circle.fill" : @"checkmark.circle.fill" accessibilityDescription:self.statusTitle.stringValue];
    self.statusIcon.contentTintColor=failed ? NSColor.systemOrangeColor : busy ? NSColor.secondaryLabelColor : setup ? NSColor.controlAccentColor : NSColor.systemGreenColor;
    [self refreshControls];
}
- (void)refreshControls {
    BOOL idle=self.running==nil;
    for(NSButton *button in self.buttons)button.enabled=idle;
    self.channel.enabled=idle;self.candidates.enabled=idle;
    self.installButton.title=self.preparation ? @"Install now" : self.updateAvailable && !self.configurationChanged ? @"Update now" : self.hasInstallation || self.legacySource ? @"Review changes" : @"Review installation";
    self.installButton.enabled=idle && (self.preparation || (self.updateAvailable && !self.configurationChanged) || (self.workspace && self.inkscape && (self.codex.state==NSControlStateValueOn || self.claude.state==NSControlStateValueOn)));
    self.cancelButton.hidden=self.preparation==nil;self.cancelButton.enabled=idle && self.preparation!=nil;
    self.checkButton.hidden=!self.hasInstallation;self.checkButton.enabled=idle && self.hasInstallation && !self.recoveryRequired;
    for(NSButton *button in self.updateButtons)button.enabled=idle && self.hasInstallation && !self.recoveryRequired;
    self.rollbackButton.enabled=idle && self.hasInstallation && self.rollbackAvailable && !self.recoveryRequired;
    self.recoverButton.hidden=!self.recoveryRequired && ![@[@"failed",@"damaged"] containsObject:self.state ?: @""];
    self.progress.hidden=idle;
    self.channelHint.stringValue=self.channel.indexOfSelectedItem==0 ? @"Stable releases are recommended for everyday use." : @"Prereleases include preview features and may be less stable.";
    NSString *hint=@"Codex uses one shared configuration for the desktop app and CLI.";
    for(NSDictionary *client in self.clientDiscovery){
        BOOL selected=[client[@"name"] isEqualToString:@"codex"] ? self.codex.state==NSControlStateValueOn : self.claude.state==NSControlStateValueOn;
        if(selected && [client[@"error"] isKindOfClass:NSString.class])hint=client[@"error"];
        else if(selected && [client[@"name"] isEqualToString:@"claude"] && ![client[@"available"] boolValue])hint=@"Claude Code was not found. Install it before connecting, or deselect it.";
    }
    self.clientHint.stringValue=hint;
}
- (NSTextField *)pathLabel:(NSString *)path {
    NSTextField *label=[NSTextField labelWithString:path.stringByAbbreviatingWithTildeInPath];label.font=[NSFont systemFontOfSize:12];label.textColor=NSColor.secondaryLabelColor;label.lineBreakMode=NSLineBreakByTruncatingMiddle;label.selectable=YES;label.toolTip=path;
    [label setContentCompressionResistancePriority:250 forOrientation:NSLayoutConstraintOrientationHorizontal];return label;
}
- (void)setPath:(NSString *)path label:(NSTextField *)label {
    label.stringValue=path.stringByAbbreviatingWithTildeInPath;label.toolTip=path;
}
- (NSStackView *)pathRow:(NSTextField *)label button:(NSString *)title action:(SEL)action {
    NSButton *button=[self button:title action:action];[button setContentCompressionResistancePriority:NSLayoutPriorityRequired forOrientation:NSLayoutConstraintOrientationHorizontal];
    return managerStack(@[label,managerSpacer(),button],NSUserInterfaceLayoutOrientationHorizontal,12);
}
- (void)addSection:(NSString *)title content:(NSView *)view to:(NSStackView *)stack {
    NSTextField *heading=managerLabel(title,13,NSColor.labelColor);heading.font=[NSFont systemFontOfSize:13 weight:NSFontWeightSemibold];
    NSStackView *body=managerStack(@[heading,view],NSUserInterfaceLayoutOrientationVertical,10);managerFill(view,body);
    NSBox *card=managerCard(body);[stack addArrangedSubview:card];managerFill(card,stack);
}
- (void)toggleAdvanced:(id)sender {self.advancedCard.hidden=self.advancedButton.state!=NSControlStateValueOn;}
- (void)buildWindow {
    self.installation=[[NSUserDefaults standardUserDefaults] stringForKey:@"InstallationDirectory"] ?: [managerHome() stringByAppendingPathComponent:@"Library/Application Support/inkscape-mcp"];
    if(![[NSFileManager defaultManager] fileExistsAtPath:[[NSBundle mainBundle].resourcePath stringByAppendingPathComponent:@"offline-runtime.tar.gz"]]){
        NSString *legacyRoot=[[NSBundle mainBundle].bundlePath stringByDeletingLastPathComponent].stringByDeletingLastPathComponent;
        if([[NSFileManager defaultManager] fileExistsAtPath:[legacyRoot stringByAppendingPathComponent:@"FILES.json"]])[[NSUserDefaults standardUserDefaults] setObject:legacyRoot forKey:@"BundledLegacyPackageDirectory"];
    }
    self.buttons=[NSMutableArray new];self.updateButtons=[NSMutableArray new];
    self.window=[[NSWindow alloc] initWithContentRect:NSMakeRect(0,0,680,740) styleMask:NSWindowStyleMaskTitled|NSWindowStyleMaskClosable|NSWindowStyleMaskMiniaturizable|NSWindowStyleMaskResizable backing:NSBackingStoreBuffered defer:NO];
    self.window.title=@"Inkscape MCP Manager";self.window.delegate=self;self.window.contentMinSize=NSMakeSize(620,600);self.window.backgroundColor=NSColor.windowBackgroundColor;
    NSView *content=self.window.contentView;
    NSImageView *icon=[NSImageView imageViewWithImage:NSApp.applicationIconImage];icon.imageScaling=NSImageScaleProportionallyUpOrDown;
    [icon.widthAnchor constraintEqualToConstant:48].active=YES;[icon.heightAnchor constraintEqualToConstant:48].active=YES;
    NSTextField *title=managerLabel(@"Inkscape MCP",24,NSColor.labelColor);title.font=[NSFont systemFontOfSize:24 weight:NSFontWeightSemibold];
    self.summary=managerLabel(@"Connect your SVG workspace to your AI tools.",12,NSColor.secondaryLabelColor);
    NSStackView *header=managerStack(@[icon,managerStack(@[title,self.summary],NSUserInterfaceLayoutOrientationVertical,4)],NSUserInterfaceLayoutOrientationHorizontal,14);
    header.translatesAutoresizingMaskIntoConstraints=NO;[content addSubview:header];
    NSScrollView *scroll=[NSScrollView new];scroll.drawsBackground=NO;scroll.hasVerticalScroller=YES;scroll.autohidesScrollers=YES;scroll.translatesAutoresizingMaskIntoConstraints=NO;[content addSubview:scroll];
    NSView *document=[ManagerDocumentView new];document.translatesAutoresizingMaskIntoConstraints=NO;scroll.documentView=document;
    NSStackView *stack=managerStack(@[],NSUserInterfaceLayoutOrientationVertical,12);stack.translatesAutoresizingMaskIntoConstraints=NO;[document addSubview:stack];
    self.installed=managerLabel(@"No runtime installed yet.",11,NSColor.secondaryLabelColor);self.installed.selectable=YES;
    self.available=managerLabel(@"The bundled version is ready to install offline.",12,NSColor.secondaryLabelColor);
    self.status=managerLabel(@"Checking this installation…",12,NSColor.secondaryLabelColor);
    self.status.maximumNumberOfLines=4;self.status.lineBreakMode=NSLineBreakByTruncatingTail;self.status.selectable=YES;
    self.installationLabel=[self pathLabel:self.installation];
    self.candidates=[NSPopUpButton new];[self.candidates addItemWithTitle:@"New installation"];
    self.codex=[self button:@"Codex (app + CLI)" action:@selector(changeOptions:)];self.codex.buttonType=NSButtonTypeSwitch;self.codex.state=NSControlStateValueOn;
    self.claude=[self button:@"Claude Code" action:@selector(changeOptions:)];self.claude.buttonType=NSButtonTypeSwitch;
    self.skills=[self button:@"Install agent skill" action:@selector(changeOptions:)];self.skills.buttonType=NSButtonTypeSwitch;self.skills.state=NSControlStateValueOn;
    self.clientHint=managerLabel(@"Codex uses one shared configuration for the desktop app and CLI.",11,NSColor.secondaryLabelColor);
    NSStackView *clients=managerStack(@[managerStack(@[self.codex,self.claude],NSUserInterfaceLayoutOrientationHorizontal,24),self.clientHint,self.skills],NSUserInterfaceLayoutOrientationVertical,10);
    managerFill(self.clientHint,clients);[self addSection:@"Connect your tools" content:clients to:stack];
    self.workspaceLabel=[self pathLabel:@"Choose the folder containing your SVG drawings."];
    [self addSection:@"SVG workspace" content:[self pathRow:self.workspaceLabel button:@"Choose folder…" action:@selector(chooseWorkspace:)] to:stack];
    self.inkscapeLabel=[self pathLabel:@"Looking for Inkscape 1.4 or newer…"];
    [self addSection:@"Inkscape" content:[self pathRow:self.inkscapeLabel button:@"Choose app…" action:@selector(chooseInkscape:)] to:stack];
    self.channel=[NSPopUpButton new];[self.channel addItemsWithTitles:@[@"Stable releases",@"Prereleases"]];
    if([[[NSBundle mainBundle] objectForInfoDictionaryKey:@"InkscapeMCPChannel"] isEqualToString:@"prerelease"])[self.channel selectItemAtIndex:1];
    self.channel.target=self;self.channel.action=@selector(changeOptions:);
    self.previewChannelButton=[self button:@"Use prereleases" action:@selector(usePrereleases:)];self.previewChannelButton.hidden=YES;
    self.channelHint=managerLabel(@"",11,NSColor.secondaryLabelColor);
    NSStackView *updates=managerStack(@[managerStack(@[self.channel,self.previewChannelButton],NSUserInterfaceLayoutOrientationHorizontal,10),self.channelHint,self.available],NSUserInterfaceLayoutOrientationVertical,8);
    managerFill(self.channelHint,updates);managerFill(self.available,updates);[self addSection:@"Updates" content:updates to:stack];
    self.advancedButton=[self button:@"Advanced" action:@selector(toggleAdvanced:)];self.advancedButton.bezelStyle=NSBezelStyleDisclosure;self.advancedButton.buttonType=NSButtonTypePushOnPushOff;
    self.advancedButton.accessibilityLabel=@"Advanced settings";
    [stack addArrangedSubview:managerStack(@[self.advancedButton,managerLabel(@"Advanced",12,NSColor.labelColor),managerLabel(@"Installation, versions & recovery",11,NSColor.secondaryLabelColor)],NSUserInterfaceLayoutOrientationHorizontal,8)];
    [self.candidates setContentCompressionResistancePriority:250 forOrientation:NSLayoutConstraintOrientationHorizontal];self.candidates.cell.lineBreakMode=NSLineBreakByTruncatingMiddle;
    NSStackView *found=managerStack(@[self.candidates,[self button:@"Use selection" action:@selector(selectCandidate:)],[self button:@"Choose installation…" action:@selector(choose:)]],NSUserInterfaceLayoutOrientationHorizontal,8);
    NSButton *instructions=[self button:@"Instructions only" action:@selector(instructions:)];NSButton *runtime=[self button:@"Runtime only" action:@selector(runtime:)];
    self.rollbackButton=[self button:@"Roll back" action:@selector(rollback:)];[self.updateButtons addObjectsFromArray:@[instructions,runtime]];
    NSStackView *advancedContent=managerStack(@[managerLabel(@"MCP installation",12,NSColor.labelColor),self.installationLabel,found,self.installed,managerStack(@[instructions,runtime,self.rollbackButton],NSUserInterfaceLayoutOrientationHorizontal,8),managerLabel(@"Your drawings and unrelated client settings are preserved. New monitoring is off.",11,NSColor.secondaryLabelColor)],NSUserInterfaceLayoutOrientationVertical,12);
    managerFill(self.installed,advancedContent);managerFill(self.installationLabel,advancedContent);managerFill(found,advancedContent);
    self.advancedCard=managerCard(advancedContent);self.advancedCard.hidden=YES;[stack addArrangedSubview:self.advancedCard];managerFill(self.advancedCard,stack);
    self.statusTitle=managerLabel(@"Checking installation",12,NSColor.labelColor);self.statusTitle.font=[NSFont systemFontOfSize:12 weight:NSFontWeightSemibold];
    self.statusIcon=[NSImageView new];[self.statusIcon.widthAnchor constraintEqualToConstant:18].active=YES;[self.statusIcon.heightAnchor constraintEqualToConstant:18].active=YES;
    self.progress=[NSProgressIndicator new];self.progress.style=NSProgressIndicatorStyleSpinning;self.progress.controlSize=NSControlSizeSmall;self.progress.indeterminate=YES;[self.progress.widthAnchor constraintEqualToConstant:16].active=YES;
    NSStackView *statusText=managerStack(@[self.statusTitle,self.status],NSUserInterfaceLayoutOrientationVertical,4);managerFill(self.status,statusText);
    NSStackView *statusRow=managerStack(@[self.statusIcon,statusText,self.progress],NSUserInterfaceLayoutOrientationHorizontal,10);
    NSBox *statusCard=managerCard(statusRow);statusCard.translatesAutoresizingMaskIntoConstraints=NO;[content addSubview:statusCard];
    self.installButton=[self button:@"Review installation" action:@selector(install:)];self.installButton.keyEquivalent=@"\r";
    self.checkButton=[self button:@"Check updates" action:@selector(check:)];self.cancelButton=[self button:@"Cancel review" action:@selector(cancelPrepared:)];self.recoverButton=[self button:@"Recover / retry" action:@selector(recover:)];
    NSStackView *actions=managerStack(@[self.recoverButton,self.cancelButton,managerSpacer(),self.checkButton,self.installButton],NSUserInterfaceLayoutOrientationHorizontal,10);actions.translatesAutoresizingMaskIntoConstraints=NO;[content addSubview:actions];
    [NSLayoutConstraint activateConstraints:@[
        [header.leadingAnchor constraintEqualToAnchor:content.leadingAnchor constant:24],[header.topAnchor constraintEqualToAnchor:content.topAnchor constant:22],[header.trailingAnchor constraintEqualToAnchor:content.trailingAnchor constant:-24],
        [scroll.leadingAnchor constraintEqualToAnchor:content.leadingAnchor constant:24],[scroll.trailingAnchor constraintEqualToAnchor:content.trailingAnchor constant:-24],[scroll.topAnchor constraintEqualToAnchor:header.bottomAnchor constant:22],[scroll.bottomAnchor constraintEqualToAnchor:statusCard.topAnchor constant:-16],
        [document.widthAnchor constraintEqualToAnchor:scroll.contentView.widthAnchor],[stack.leadingAnchor constraintEqualToAnchor:document.leadingAnchor],[stack.trailingAnchor constraintEqualToAnchor:document.trailingAnchor],[stack.topAnchor constraintEqualToAnchor:document.topAnchor],[stack.bottomAnchor constraintEqualToAnchor:document.bottomAnchor],
        [statusCard.leadingAnchor constraintEqualToAnchor:content.leadingAnchor constant:24],[statusCard.trailingAnchor constraintEqualToAnchor:content.trailingAnchor constant:-24],[statusCard.bottomAnchor constraintEqualToAnchor:actions.topAnchor constant:-14],
        [actions.leadingAnchor constraintEqualToAnchor:content.leadingAnchor constant:24],[actions.trailingAnchor constraintEqualToAnchor:content.trailingAnchor constant:-24],[actions.bottomAnchor constraintEqualToAnchor:content.bottomAnchor constant:-20]
    ]];
    [self setPhase:@"loading" message:@"Checking installed files and client settings…"];
}
- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    [self buildWindow];[self.window center];[self.window makeKeyAndOrderFront:nil];[NSApp activateIgnoringOtherApps:YES];[self run:@[@"manager-prepare"] request:nil];
}
- (BOOL)windowShouldClose:(NSWindow *)sender {if(self.running){self.status.stringValue=@"Wait for the current operation to finish before closing.";return NO;}return YES;}
- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {return YES;}
- (NSApplicationTerminateReply)applicationShouldTerminate:(NSApplication *)sender {if(self.running){self.status.stringValue=@"Wait for the current operation to finish before closing.";return NSTerminateCancel;}return NSTerminateNow;}
- (void)inspect {[self setPhase:@"loading" message:@"Checking installed files and client settings…"];[self run:@[@"install-inspect"] request:nil];}
- (void)cancelPrepared:(id)sender {if(self.running)return;self.preparation=nil;self.cancelButton.enabled=NO;[self inspect];}
- (void)changeOptions:(id)sender {
    self.preparation=nil;self.configurationChanged=YES;
    if(sender==self.channel){self.updateAvailable=NO;self.available.stringValue=self.hasInstallation ? @"Check for updates on the selected channel." : @"The bundled version is ready to install offline.";}
    if(!self.recoveryRequired)[self setPhase:@"ready" message:self.codex.state!=NSControlStateValueOn && self.claude.state!=NSControlStateValueOn ? @"Select at least one client to continue." : !self.workspace ? @"Choose an SVG workspace to continue. Installation works offline." : !self.inkscape ? @"Choose Inkscape to continue." : @"Review your choices to verify the MCP connection before installing."];
    else [self refreshControls];
}
- (void)choose:(id)sender {
    NSOpenPanel *panel=[NSOpenPanel openPanel];panel.canChooseDirectories=YES;panel.canChooseFiles=NO;panel.allowsMultipleSelection=NO;panel.message=@"Advanced recovery: choose a permanent installation or legacy source folder.";
    if([panel runModal]==NSModalResponseOK){[self selectPath:panel.URL.path];}
}
- (void)selectPath:(NSString *)path {
    if([[NSFileManager defaultManager] fileExistsAtPath:[path stringByAppendingPathComponent:@".inkscape-mcp-local/setup.conf"]]){self.legacySource=path;self.installButton.title=@"Upgrade existing installation";[self run:@[@"install-inspect",@"--source",path] request:nil];}
    else {self.legacySource=nil;self.installation=path;[[NSUserDefaults standardUserDefaults] setObject:path forKey:@"InstallationDirectory"];[self inspect];}
    [self setPath:self.installation label:self.installationLabel];
    self.preparation=nil;
}
- (void)selectCandidate:(id)sender {
    NSString *path=self.candidates.selectedItem.representedObject;
    if(path)[self selectPath:path];
    else [self selectPath:[managerHome() stringByAppendingPathComponent:@"Library/Application Support/inkscape-mcp"]];
}
- (void)chooseWorkspace:(id)sender {
    NSOpenPanel *panel=[NSOpenPanel openPanel];panel.canChooseDirectories=YES;panel.canChooseFiles=NO;panel.allowsMultipleSelection=NO;panel.message=@"Choose the existing folder for your SVG drawings.";
    if([panel runModal]==NSModalResponseOK){self.workspace=panel.URL.path;[self setPath:self.workspace label:self.workspaceLabel];[self changeOptions:sender];}
}
- (void)chooseInkscape:(id)sender {
    NSOpenPanel *panel=[NSOpenPanel openPanel];panel.canChooseDirectories=NO;panel.canChooseFiles=YES;panel.allowsMultipleSelection=NO;panel.message=@"Choose Inkscape.app, or its inkscape executable.";
    if([panel runModal]==NSModalResponseOK){NSString *path=panel.URL.path;if([path.pathExtension isEqualToString:@"app"])path=[path stringByAppendingPathComponent:@"Contents/MacOS/inkscape"];self.inkscape=path;[self setPath:path label:self.inkscapeLabel];[self changeOptions:sender];}
}
- (NSString *)packageRoot {
    return [[NSUserDefaults standardUserDefaults] stringForKey:@"BundledLegacyPackageDirectory"] ?: [[NSBundle mainBundle].bundlePath stringByDeletingLastPathComponent].stringByDeletingLastPathComponent;
}
- (void)install:(id)sender {
    if(self.updateAvailable && !self.configurationChanged && !self.preparation){[self both:sender];return;}
    if(self.preparation){[self run:@[@"install-activate",@"--preparation",self.preparation] request:nil];return;}
    if(!self.workspace || !self.inkscape){[self setPhase:@"failed" message:@"Choose an SVG workspace and Inkscape before installing."];return;}
    NSMutableArray *clients=[NSMutableArray new];if(self.codex.state==NSControlStateValueOn)[clients addObject:@"codex"];if(self.claude.state==NSControlStateValueOn)[clients addObject:@"claude"];
    if(!clients.count){[self setPhase:@"failed" message:@"Choose Codex, Claude Code, or both."];return;}
    NSString *resources=[NSBundle mainBundle].resourcePath;
    NSString *package=[resources stringByAppendingPathComponent:@"offline-runtime.tar.gz"];
    if(![[NSFileManager defaultManager] fileExistsAtPath:package])package=[self packageRoot];
    NSMutableDictionary *request=[@{@"package":package,@"instructions":[resources stringByAppendingPathComponent:@"instructions"],@"workspace":self.workspace,@"inkscape":self.inkscape,@"clients":clients,@"install_skills":(self.skills.state==NSControlStateValueOn ? @YES : @NO),@"channel":self.channel.indexOfSelectedItem==0 ? @"stable" : @"prerelease"} mutableCopy];
    if(self.legacySource)request[@"legacy_source"]=self.legacySource;
    [self run:@[@"install-prepare"] request:request];
}
- (void)check:(id)sender {[self update:@[@"--check"]];}
- (void)instructions:(id)sender {[self update:@[@"--instructions"]];}
- (void)runtime:(id)sender {[self update:@[@"--runtime"]];}
- (void)both:(id)sender {[self update:@[]];}
- (void)rollback:(id)sender {[self run:@[@"rollback"] request:nil];}
- (void)recover:(id)sender {if([[NSFileManager defaultManager] fileExistsAtPath:self.installation])[self run:@[@"install-recover"] request:nil];else [self inspect];}
- (void)update:(NSArray *)options {NSMutableArray *args=[NSMutableArray arrayWithObject:@"update"];[args addObjectsFromArray:options];[args addObjectsFromArray:@[@"--channel",self.channel.indexOfSelectedItem==0 ? @"stable" : @"prerelease"]];[self run:args request:nil];}
- (void)usePrereleases:(id)sender {[self.channel selectItemAtIndex:1];self.previewChannelButton.hidden=YES;[self changeOptions:self.channel];[self setPhase:self.state message:@"Prereleases selected. Choose Check updates to review availability, then Update now to install."];}
- (void)run:(NSArray *)arguments request:(NSDictionary *)request {
    if(self.running)return;
    NSString *executable=[[NSBundle mainBundle].bundlePath stringByAppendingPathComponent:@"Contents/Helpers/inkscape-mcp-launcher"];
    if(![[NSFileManager defaultManager] isExecutableFileAtPath:executable]){[self setPhase:@"failed" message:@"The bundled installer is missing or damaged. Download a complete Manager distribution and retry."];return;}
    NSTask *task=[NSTask new];task.executableURL=[NSURL fileURLWithPath:executable];NSMutableArray *args=[arguments mutableCopy];[args addObjectsFromArray:@[@"--install-dir",self.installation,@"--json"]];task.arguments=args;
    NSPipe *input=request ? [NSPipe pipe] : nil;task.standardInput=input ?: [NSFileHandle fileHandleWithNullDevice];NSPipe *output=[NSPipe pipe];NSPipe *errors=[NSPipe pipe];task.standardOutput=output;task.standardError=errors;
    NSError *error=nil;if(![task launchAndReturnError:&error]){[self setPhase:@"failed" message:error.localizedDescription];return;}
    self.running=task;[self.progress startAnimation:nil];[self setPhase:@"busy" message:[arguments containsObject:@"install-prepare"] ? @"Checking prerequisites, preparing files and checking the MCP connection…" : @"Verifying and applying the selected operation…"];
    dispatch_group_t group=dispatch_group_create();__block NSData *out=nil;__block NSData *err=nil;
    __block BOOL timedOut=NO;
    NSTimeInterval deadline=NSProcessInfo.processInfo.systemUptime+602;
    managerTaskDeadline(task,600,2,^BOOL{return self.running==task;},^{timedOut=YES;});
    if(request){NSData *data=[NSJSONSerialization dataWithJSONObject:request options:0 error:nil];dispatch_group_async(group,dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0),^{managerWriteInput(input.fileHandleForWriting,data,deadline);});}
    dispatch_group_async(group,dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0),^{out=managerReadOutput(output.fileHandleForReading,task,deadline,nil);});
    dispatch_group_async(group,dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0),^{err=managerReadOutput(errors.fileHandleForReading,task,deadline,^(NSData *line){
        NSDictionary *event=[NSJSONSerialization JSONObjectWithData:line options:0 error:nil];
        if(![event isKindOfClass:NSDictionary.class] || ![event[@"event"] isEqualToString:@"progress"])return;
        dispatch_async(dispatch_get_main_queue(),^{if(self.running!=task)return;
            if([event[@"stage"] isEqualToString:@"download"] && [event[@"received_bytes"] isKindOfClass:NSNumber.class]){
                self.status.stringValue=[event[@"total_bytes"] isKindOfClass:NSNumber.class] ? [NSString stringWithFormat:@"Downloaded %@ of %@ bytes…",event[@"received_bytes"],event[@"total_bytes"]] : [NSString stringWithFormat:@"Downloaded %@ bytes…",event[@"received_bytes"]];
            }else if([@[@"prerequisites",@"prepare",@"probe",@"activate",@"complete"] containsObject:event[@"stage"]] && [event[@"message"] isKindOfClass:NSString.class])self.status.stringValue=event[@"message"];
        });
    });});
    dispatch_group_async(group,dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0),^{[task waitUntilExit];});
    dispatch_group_notify(group,dispatch_get_main_queue(),^{
        self.running=nil;[self.progress stopAnimation:nil];[self refreshControls];
        if(timedOut){self.preparation=nil;self.cancelButton.enabled=NO;[self setPhase:@"failed" message:@"The operation timed out. Use Recover / retry before trying again."];return;}
        NSDictionary *result=out.length ? [NSJSONSerialization JSONObjectWithData:out options:0 error:nil] : nil;
        if(![result isKindOfClass:NSDictionary.class]){[self setPhase:@"failed" message:err.length ? [[NSString alloc] initWithData:err encoding:NSUTF8StringEncoding] : @"Operation interrupted. Use Recover / retry before trying again."];return;}
        self.previewChannelButton.hidden=YES;
        if(task.terminationStatus!=0){
            if([arguments containsObject:@"update"] && self.channel.indexOfSelectedItem==0 && [result[@"error"] isEqualToString:@"no compatible update release found on the selected channel"]){self.previewChannelButton.hidden=NO;[self setPhase:@"installed" message:@"Stable releases currently have no eligible updates. Stay on Stable, or choose Use prereleases to check preview versions."];return;}
            [self setPhase:@"failed" message:result[@"error"] ?: @"Operation failed. Review the prerequisites and retry or recover."];return;}
        if([arguments containsObject:@"manager-prepare"]){
            if([result[@"manager_ready"] boolValue]){[self inspect];return;}
            NSAlert *review=[NSAlert new];review.messageText=[result[@"replaces_existing_manager"] boolValue] ? @"Upgrade the Manager application?" : @"Install the Manager application?";
            review.informativeText=[NSString stringWithFormat:@"The verified application will be installed at %@ and reopened there. Existing runtime settings and drawings are preserved. A running destination Manager must be closed first.",result[@"manager_application"]];
            [review addButtonWithTitle:@"Install and reopen"];[review addButtonWithTitle:@"Cancel"];
            if([review runModal]!=NSAlertFirstButtonReturn){[self setPhase:@"not_installed" message:@"Manager installation cancelled. Reopen the distribution to retry."];return;}
            NSString *helper=[result[@"manager_stage"] stringByAppendingPathComponent:@"Contents/Helpers/inkscape-mcp-launcher"];
            NSTask *relocate=[NSTask new];relocate.executableURL=[NSURL fileURLWithPath:helper];relocate.arguments=@[@"manager-activate",@"--preparation",result[@"preparation_id"],@"--parent-pid",[NSString stringWithFormat:@"%d",getpid()],@"--install-dir",self.installation,@"--json"];
            relocate.standardInput=[NSFileHandle fileHandleWithNullDevice];relocate.standardOutput=[NSFileHandle fileHandleWithNullDevice];relocate.standardError=[NSFileHandle fileHandleWithNullDevice];
            NSError *launchError=nil;if(![relocate launchAndReturnError:&launchError]){[self setPhase:@"failed" message:launchError.localizedDescription];return;}
            [NSApp terminate:nil];return;
        }
        if([arguments containsObject:@"install-inspect"]){[self applyInspection:result];return;}
        if(result[@"preparation_id"]){self.preparation=result[@"preparation_id"];[self setPhase:@"prepared" message:[NSString stringWithFormat:@"The MCP connection passed. Install into %@? Your drawings and client preferences are preserved.%@",self.installation.stringByAbbreviatingWithTildeInPath,[result[@"skipped_skills"] count] ? @" User-owned skills will be left untouched." : @""]];return;}
        NSInteger requestedChannel=self.channel.indexOfSelectedItem;
        if(result[@"runtime_build"])[self showInstalled:result];
        if([arguments containsObject:@"--check"])[self.channel selectItemAtIndex:requestedChannel];
        NSDictionary *available=result[@"available"];if(available)self.available.stringValue=[NSString stringWithFormat:@"Available: %@",available[@"distribution"]];
        if([arguments containsObject:@"--check"])self.updateAvailable=[result[@"update_available"] boolValue];
        if([result[@"state"] isEqualToString:@"completed"] && [result[@"changed"] isEqual:@NO]){self.preparation=nil;self.configurationChanged=NO;self.updateAvailable=NO;[self setPhase:@"completed" message:@"This installation is already up to date. No client restart is needed."];}
        else if([result[@"client_reconnect_required"] boolValue]){self.preparation=nil;self.configurationChanged=NO;self.updateAvailable=NO;[self setPhase:@"completed" message:@"Restart Codex / Claude Code to load the update and its agent skill."];}
        else if([arguments containsObject:@"--check"])[self setPhase:@"installed" message:[result[@"update_available"] boolValue] ? @"An update is available. Choose Update, or an individual component." : @"No update is available for the selected channel."];
        else if([arguments containsObject:@"install-recover"]){self.preparation=nil;[self inspect];}
        else [self setPhase:@"installed" message:@"Ready. Updates run when you choose an action."];
        NSArray *skipped=result[@"skipped_skills"];if(skipped.count)self.status.stringValue=[self.status.stringValue stringByAppendingString:@" Skipped skills were preserved."];
    });
}
- (void)applyInspection:(NSDictionary *)result {
    self.hasInstallation=result[@"installed"]!=nil;self.recoveryRequired=[result[@"recovery_required"] boolValue];self.configurationChanged=NO;self.updateAvailable=NO;self.clientDiscovery=result[@"clients"];
    NSDictionary *installed=result[@"installed"];
    if(installed)[self showInstalled:installed];
    else {self.installed.stringValue=@"No verified runtime installed in this folder.";self.summary.stringValue=[result[@"state"] isEqualToString:@"legacy_found"] ? @"An existing installation is ready to upgrade." : @"Connect your SVG workspace to your AI tools.";}
    [self.candidates removeAllItems];[self.candidates addItemWithTitle:@"New installation"];
    for(NSString *path in result[@"candidates"]){[self.candidates addItemWithTitle:path];self.candidates.lastItem.representedObject=path;if([path isEqualToString:self.legacySource ?: self.installation])[self.candidates selectItem:self.candidates.lastItem];}
    if([result[@"workspace"] isKindOfClass:NSString.class] && ![result[@"workspace"] containsString:@":"]){self.workspace=result[@"workspace"];[self setPath:self.workspace label:self.workspaceLabel];}
    if([result[@"inkscape"] isKindOfClass:NSString.class]){self.inkscape=result[@"inkscape"];[self setPath:self.inkscape label:self.inkscapeLabel];}
    else {self.inkscape=nil;self.inkscapeLabel.stringValue=@"Inkscape 1.4+ is required. Choose the installed app.";self.inkscapeLabel.toolTip=nil;}
    NSString *message=self.hasInstallation ? @"Check for updates, or review changes to your setup." : !self.workspace ? @"Choose an SVG workspace to continue. Installation works offline." : !self.inkscape ? @"Choose Inkscape to continue. Your drawings will stay in their original folder." : @"Review your choices to verify the MCP connection before installing.";
    if([installed[@"bootstrap_damaged"] boolValue])message=@"The permanent launcher needs repair. Review the bundled installation to restore it.";
    else if(self.recoveryRequired)message=@"An interrupted installation needs recovery. Choose Recover / retry.";
    else if([result[@"state"] isEqualToString:@"damaged"])message=@"This installation could not be verified. Recover it or select another installation in Advanced.";
    else if([result[@"choice_required"] boolValue] && !self.legacySource){message=@"Multiple installations were found. Choose the one to upgrade in Advanced.";self.advancedButton.state=NSControlStateValueOn;self.advancedCard.hidden=NO;}
    [self setPhase:self.recoveryRequired || [result[@"state"] isEqualToString:@"damaged"] ? @"damaged" : result[@"state"] ?: @"not_installed" message:message];
}
- (void)showInstalled:(NSDictionary *)result {
    self.hasInstallation=YES;self.rollbackAvailable=[result[@"rollback_available"] boolValue];
    self.summary.stringValue=[NSString stringWithFormat:@"Installed · %@",result[@"distribution"] ?: @"Current runtime"];
    NSString *manager=[[NSBundle mainBundle] objectForInfoDictionaryKey:@"InkscapeMCPBuildID"] ?: @"unknown";
    NSString *managerDistribution=[[NSBundle mainBundle] objectForInfoDictionaryKey:@"InkscapeMCPDistribution"] ?: @"development";
    NSDictionary *signature=result[@"manager_helper_signing"];
    NSString *signing=[signature[@"status"] isEqualToString:@"developer_id_verified"] ? [NSString stringWithFormat:@"Developer ID verified (%@)",signature[@"team_id"]] : [signature[@"status"] isEqualToString:@"ad_hoc"] ? @"Development (ad-hoc)" : @"Not verified";
    self.installed.stringValue=[NSString stringWithFormat:@"Distribution: %@ · Manager distribution: %@\nRuntime: %@\nInstructions: %@\nManager: %@\nLauncher: %@ · Compatibility: %@\nManager helper signing: %@ · Notarization: not checked on this Mac",result[@"distribution"],managerDistribution,result[@"runtime_build"],result[@"instructions_version"],manager,[result[@"launcher_build"] isKindOfClass:NSString.class] ? result[@"launcher_build"] : @"unknown (upgrade to record)",result[@"launcher_version"],signing];
    if(result[@"channel"])[self.channel selectItemAtIndex:[result[@"channel"] isEqualToString:@"stable"] ? 0 : 1];
}
@end
int main(int argc,const char **argv){@autoreleasepool{NSApplication *app=[NSApplication sharedApplication];Manager *delegate=[Manager new];app.delegate=delegate;[app setActivationPolicy:NSApplicationActivationPolicyRegular];[app run];}return 0;}
