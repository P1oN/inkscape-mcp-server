#import <Cocoa/Cocoa.h>

static NSString *managerHome(void) {
    NSString *value=NSProcessInfo.processInfo.environment[@"HOME"];
    return value.isAbsolutePath ? value : NSHomeDirectory();
}

// Thin native UI. Discovery, verification, preservation and recovery belong to Rust.
@interface Manager : NSObject <NSApplicationDelegate, NSWindowDelegate>
@property NSWindow *window;
@property NSTextField *installed;
@property NSTextField *available;
@property NSTextField *status;
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
@end
@implementation Manager
- (NSButton *)button:(NSString *)title action:(SEL)action {
    NSButton *button=[NSButton buttonWithTitle:title target:self action:action];
    [self.buttons addObject:button]; return button;
}
- (void)setPhase:(NSString *)phase message:(NSString *)message {
    self.state=phase; self.status.stringValue=message;
    if([phase isEqualToString:@"loading"])self.installed.stringValue=@"Reading installed versions…";
    if([phase isEqualToString:@"not_installed"] && [self.installed.stringValue hasPrefix:@"Reading installed versions"] )self.installed.stringValue=@"Inkscape MCP is not installed";
    if([phase isEqualToString:@"failed"])self.installed.stringValue=@"The operation did not complete. Review the message below and retry or recover.";
}
- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    self.installation=[[NSUserDefaults standardUserDefaults] stringForKey:@"InstallationDirectory"] ?: [managerHome() stringByAppendingPathComponent:@"Library/Application Support/inkscape-mcp"];
    if(![[NSFileManager defaultManager] fileExistsAtPath:[[NSBundle mainBundle].resourcePath stringByAppendingPathComponent:@"offline-runtime.tar.gz"]]){
        NSString *legacyRoot=[[NSBundle mainBundle].bundlePath stringByDeletingLastPathComponent].stringByDeletingLastPathComponent;
        if([[NSFileManager defaultManager] fileExistsAtPath:[legacyRoot stringByAppendingPathComponent:@"FILES.json"]])[[NSUserDefaults standardUserDefaults] setObject:legacyRoot forKey:@"BundledLegacyPackageDirectory"];
    }
    self.buttons=[NSMutableArray new];
    self.window=[[NSWindow alloc] initWithContentRect:NSMakeRect(0,0,720,760) styleMask:NSWindowStyleMaskTitled|NSWindowStyleMaskClosable|NSWindowStyleMaskMiniaturizable backing:NSBackingStoreBuffered defer:NO];
    self.window.title=@"Inkscape MCP Manager";self.window.delegate=self;
    NSStackView *stack=[NSStackView new];stack.orientation=NSUserInterfaceLayoutOrientationVertical;stack.alignment=NSLayoutAttributeLeading;stack.spacing=14;stack.edgeInsets=NSEdgeInsetsMake(24,24,24,24);
    self.installed=[NSTextField wrappingLabelWithString:@"Reading installed versions…"];
    self.available=[NSTextField wrappingLabelWithString:@"The bundled version can be installed offline. Check online for newer versions when ready."];
    self.status=[NSTextField wrappingLabelWithString:@"Checking this installation…"];
    [stack addArrangedSubview:self.installed];[stack addArrangedSubview:self.available];
    self.candidates=[NSPopUpButton new];[self.candidates addItemWithTitle:@"New installation"];
    NSStackView *found=[NSStackView stackViewWithViews:@[self.candidates,[self button:@"Use selected installation" action:@selector(selectCandidate:)],[self button:@"Choose installation…" action:@selector(choose:)]]];found.spacing=8;[stack addArrangedSubview:found];
    self.codex=[self button:@"Codex" action:@selector(changeOptions:)];self.codex.buttonType=NSButtonTypeSwitch;self.codex.state=NSControlStateValueOn;
    self.claude=[self button:@"Claude Code" action:@selector(changeOptions:)];self.claude.buttonType=NSButtonTypeSwitch;
    self.skills=[self button:@"Install agent skill" action:@selector(changeOptions:)];self.skills.buttonType=NSButtonTypeSwitch;self.skills.state=NSControlStateValueOn;
    NSStackView *clients=[NSStackView stackViewWithViews:@[[NSTextField labelWithString:@"Connect"],self.codex,self.claude,self.skills]];clients.spacing=12;[stack addArrangedSubview:clients];
    self.workspaceLabel=[NSTextField wrappingLabelWithString:@"Choose the folder containing your SVG drawings."];
    [stack addArrangedSubview:[NSStackView stackViewWithViews:@[[self button:@"SVG workspace…" action:@selector(chooseWorkspace:)],self.workspaceLabel]]];
    self.inkscapeLabel=[NSTextField wrappingLabelWithString:@"Checking for Inkscape 1.4 or newer…"];
    [stack addArrangedSubview:[NSStackView stackViewWithViews:@[[self button:@"Choose Inkscape…" action:@selector(chooseInkscape:)],self.inkscapeLabel]]];
    self.channel=[NSPopUpButton new];[self.channel addItemsWithTitles:@[@"Stable releases",@"Prereleases"]];
    if([[[NSBundle mainBundle] objectForInfoDictionaryKey:@"InkscapeMCPChannel"] isEqualToString:@"prerelease"])[self.channel selectItemAtIndex:1];
    self.channel.target=self;self.channel.action=@selector(changeOptions:);
    self.previewChannelButton=[self button:@"Use prereleases" action:@selector(usePrereleases:)];self.previewChannelButton.hidden=YES;
    [stack addArrangedSubview:[NSStackView stackViewWithViews:@[[NSTextField labelWithString:@"Update channel"],self.channel,self.previewChannelButton]]];
    [stack addArrangedSubview:[NSTextField wrappingLabelWithString:@"Prereleases are preview versions. New monitoring is off. Existing workspace and monitoring choices are preserved during upgrade."]];
    self.installButton=[self button:@"Install" action:@selector(install:)];
    self.cancelButton=[self button:@"Cancel prepared install" action:@selector(cancelPrepared:)];self.cancelButton.enabled=NO;
    NSStackView *actions=[NSStackView stackViewWithViews:@[self.installButton,[self button:@"Check updates" action:@selector(check:)],[self button:@"Update" action:@selector(both:)]]];actions.spacing=12;[stack addArrangedSubview:actions];[stack addArrangedSubview:self.cancelButton];
    NSStackView *components=[NSStackView stackViewWithViews:@[[self button:@"Instructions only" action:@selector(instructions:)],[self button:@"Runtime only" action:@selector(runtime:)],[self button:@"Roll back" action:@selector(rollback:)],[self button:@"Recover / retry" action:@selector(recover:)]]];components.spacing=8;[stack addArrangedSubview:components];
    self.progress=[NSProgressIndicator new];self.progress.style=NSProgressIndicatorStyleBar;self.progress.indeterminate=YES;[stack addArrangedSubview:self.progress];[stack addArrangedSubview:self.status];
    stack.translatesAutoresizingMaskIntoConstraints=NO;[self.window.contentView addSubview:stack];
    [NSLayoutConstraint activateConstraints:@[[stack.leadingAnchor constraintEqualToAnchor:self.window.contentView.leadingAnchor],[stack.trailingAnchor constraintEqualToAnchor:self.window.contentView.trailingAnchor],[stack.topAnchor constraintEqualToAnchor:self.window.contentView.topAnchor],[self.progress.widthAnchor constraintEqualToConstant:660]]];
    [self.window center];[self.window makeKeyAndOrderFront:nil];[NSApp activateIgnoringOtherApps:YES];[self run:@[@"manager-prepare"] request:nil];
}
- (BOOL)windowShouldClose:(NSWindow *)sender {if(self.running){self.status.stringValue=@"Wait for the current operation to finish before closing.";return NO;}return YES;}
- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {return YES;}
- (NSApplicationTerminateReply)applicationShouldTerminate:(NSApplication *)sender {if(self.running){self.status.stringValue=@"Wait for the current operation to finish before closing.";return NSTerminateCancel;}return NSTerminateNow;}
- (void)inspect {[self setPhase:@"loading" message:@"Checking installed files and client settings…"];[self run:@[@"install-inspect"] request:nil];}
- (void)cancelPrepared:(id)sender {if(self.running)return;self.preparation=nil;self.cancelButton.enabled=NO;[self inspect];}
- (void)changeOptions:(id)sender {self.preparation=nil;self.cancelButton.enabled=NO;self.installButton.title=self.legacySource ? @"Upgrade existing installation" : @"Install / upgrade";}
- (void)choose:(id)sender {
    NSOpenPanel *panel=[NSOpenPanel openPanel];panel.canChooseDirectories=YES;panel.canChooseFiles=NO;panel.allowsMultipleSelection=NO;panel.message=@"Advanced recovery: choose a permanent installation or legacy source folder.";
    if([panel runModal]==NSModalResponseOK){[self selectPath:panel.URL.path];}
}
- (void)selectPath:(NSString *)path {
    if([[NSFileManager defaultManager] fileExistsAtPath:[path stringByAppendingPathComponent:@".inkscape-mcp-local/setup.conf"]]){self.legacySource=path;self.installButton.title=@"Upgrade existing installation";[self run:@[@"install-inspect",@"--source",path] request:nil];}
    else {self.legacySource=nil;self.installation=path;[[NSUserDefaults standardUserDefaults] setObject:path forKey:@"InstallationDirectory"];[self inspect];}
    self.preparation=nil;
}
- (void)selectCandidate:(id)sender {
    NSString *path=self.candidates.selectedItem.representedObject;
    if(path)[self selectPath:path];
    else [self selectPath:[managerHome() stringByAppendingPathComponent:@"Library/Application Support/inkscape-mcp"]];
}
- (void)chooseWorkspace:(id)sender {
    NSOpenPanel *panel=[NSOpenPanel openPanel];panel.canChooseDirectories=YES;panel.canChooseFiles=NO;panel.allowsMultipleSelection=NO;panel.message=@"Choose the existing folder for your SVG drawings.";
    if([panel runModal]==NSModalResponseOK){self.workspace=panel.URL.path;self.workspaceLabel.stringValue=self.workspace;self.preparation=nil;self.installButton.title=self.legacySource ? @"Upgrade existing installation" : @"Install / upgrade";[self setPhase:@"not_installed" message:@"Ready. Choose Install to check this configuration."];}
}
- (void)chooseInkscape:(id)sender {
    NSOpenPanel *panel=[NSOpenPanel openPanel];panel.canChooseDirectories=NO;panel.canChooseFiles=YES;panel.allowsMultipleSelection=NO;panel.message=@"Choose Inkscape.app, or its inkscape executable.";
    if([panel runModal]==NSModalResponseOK){NSString *path=panel.URL.path;if([path.pathExtension isEqualToString:@"app"])path=[path stringByAppendingPathComponent:@"Contents/MacOS/inkscape"];self.inkscape=path;self.inkscapeLabel.stringValue=path;self.preparation=nil;}
}
- (NSString *)packageRoot {
    return [[NSUserDefaults standardUserDefaults] stringForKey:@"BundledLegacyPackageDirectory"] ?: [[NSBundle mainBundle].bundlePath stringByDeletingLastPathComponent].stringByDeletingLastPathComponent;
}
- (void)install:(id)sender {
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
- (void)usePrereleases:(id)sender {[self.channel selectItemAtIndex:1];self.previewChannelButton.hidden=YES;[self changeOptions:sender];self.status.stringValue=@"Prereleases selected. These are preview versions. Choose Check updates to review availability; installation changes only when you choose Update.";}
static NSData *boundedRead(NSFileHandle *file,NSTask *task,void (^lineHandler)(NSData *)) {
    NSMutableData *data=[NSMutableData new];NSMutableData *pending=[NSMutableData new];
    while(YES){NSData *part=[file readDataOfLength:4096];if(!part.length)break;if(data.length+part.length>1024*1024){if(task.running)[task terminate];break;}[data appendData:part];
        if(lineHandler){[pending appendData:part];while(YES){NSRange end=[pending rangeOfData:[NSData dataWithBytes:"\n" length:1] options:0 range:NSMakeRange(0,pending.length)];if(end.location==NSNotFound)break;if(end.location<=4096)lineHandler([pending subdataWithRange:NSMakeRange(0,end.location)]);[pending replaceBytesInRange:NSMakeRange(0,end.location+1) withBytes:NULL length:0];}if(pending.length>4096)[pending setLength:0];}
    }
    return data;
}
- (void)run:(NSArray *)arguments request:(NSDictionary *)request {
    if(self.running)return;
    NSString *executable=[[NSBundle mainBundle].bundlePath stringByAppendingPathComponent:@"Contents/Helpers/inkscape-mcp-launcher"];
    if(![[NSFileManager defaultManager] isExecutableFileAtPath:executable]){[self setPhase:@"failed" message:@"The bundled installer is missing or damaged. Download a complete Manager distribution and retry."];return;}
    NSTask *task=[NSTask new];task.executableURL=[NSURL fileURLWithPath:executable];NSMutableArray *args=[arguments mutableCopy];[args addObjectsFromArray:@[@"--install-dir",self.installation,@"--json"]];task.arguments=args;
    NSPipe *input=request ? [NSPipe pipe] : nil;task.standardInput=input ?: [NSFileHandle fileHandleWithNullDevice];NSPipe *output=[NSPipe pipe];NSPipe *errors=[NSPipe pipe];task.standardOutput=output;task.standardError=errors;
    NSError *error=nil;if(![task launchAndReturnError:&error]){[self setPhase:@"failed" message:error.localizedDescription];return;}
    self.running=task;for(NSButton *button in self.buttons)button.enabled=NO;self.channel.enabled=NO;self.candidates.enabled=NO;[self.progress startAnimation:nil];[self setPhase:@"busy" message:[arguments containsObject:@"install-prepare"] ? @"Checking prerequisites, preparing files and checking the MCP connection…" : @"Verifying and applying the selected operation…"];
    if(request){NSData *data=[NSJSONSerialization dataWithJSONObject:request options:0 error:nil];[input.fileHandleForWriting writeData:data];[input.fileHandleForWriting closeFile];}
    dispatch_group_t group=dispatch_group_create();__block NSData *out=nil;__block NSData *err=nil;
    dispatch_group_async(group,dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0),^{out=boundedRead(output.fileHandleForReading,task,nil);});
    dispatch_group_async(group,dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0),^{err=boundedRead(errors.fileHandleForReading,task,^(NSData *line){
        NSDictionary *event=[NSJSONSerialization JSONObjectWithData:line options:0 error:nil];
        if(![event isKindOfClass:NSDictionary.class] || ![event[@"event"] isEqualToString:@"progress"])return;
        dispatch_async(dispatch_get_main_queue(),^{if(self.running!=task)return;
            if([event[@"stage"] isEqualToString:@"download"] && [event[@"received_bytes"] isKindOfClass:NSNumber.class]){
                self.status.stringValue=[event[@"total_bytes"] isKindOfClass:NSNumber.class] ? [NSString stringWithFormat:@"Downloaded %@ of %@ bytes…",event[@"received_bytes"],event[@"total_bytes"]] : [NSString stringWithFormat:@"Downloaded %@ bytes…",event[@"received_bytes"]];
            }else if([@[@"prerequisites",@"prepare",@"probe",@"activate",@"complete"] containsObject:event[@"stage"]] && [event[@"message"] isKindOfClass:NSString.class])self.status.stringValue=event[@"message"];
        });
    });});
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW,600*NSEC_PER_SEC),dispatch_get_main_queue(),^{if(self.running==task && task.running)[task terminate];});
    dispatch_group_async(group,dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0),^{[task waitUntilExit];});
    dispatch_group_notify(group,dispatch_get_main_queue(),^{
        self.running=nil;for(NSButton *button in self.buttons)button.enabled=YES;self.cancelButton.enabled=self.preparation!=nil;self.channel.enabled=YES;self.candidates.enabled=YES;[self.progress stopAnimation:nil];
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
        if([arguments containsObject:@"install-inspect"]){
            self.state=result[@"state"] ?: @"failed";
            NSDictionary *installed=result[@"installed"];
            if(installed){[self showInstalled:installed];self.installButton.title=@"Upgrade existing installation";}
            else {self.installed.stringValue=[self.state isEqualToString:@"damaged"] ? @"Damaged installation — recovery required" : [self.state isEqualToString:@"legacy_found"] ? @"Existing installation found — select it below to upgrade" : @"Inkscape MCP is not installed";self.installButton.title=self.legacySource ? @"Upgrade existing installation" : @"Install";}
            [self.candidates removeAllItems];[self.candidates addItemWithTitle:@"New installation"];
            for(NSString *path in result[@"candidates"]){[self.candidates addItemWithTitle:path];self.candidates.lastItem.representedObject=path;if([path isEqualToString:self.legacySource ?: self.installation])[self.candidates selectItem:self.candidates.lastItem];}
            if([result[@"workspace"] isKindOfClass:NSString.class] && ![result[@"workspace"] containsString:@":"]){self.workspace=result[@"workspace"];self.workspaceLabel.stringValue=self.workspace;}
            if([result[@"inkscape"] isKindOfClass:NSString.class]){self.inkscape=result[@"inkscape"];self.inkscapeLabel.stringValue=self.inkscape;}else self.inkscapeLabel.stringValue=@"Inkscape was not found. Install Inkscape 1.4 or newer, then choose it here.";
            NSMutableArray *missing=[NSMutableArray new];
            for(NSDictionary *client in result[@"clients"]){if(![client[@"available"] boolValue])[missing addObject:[client[@"name"] isEqualToString:@"codex"] ? @"Codex" : @"Claude Code"];}
            self.status.stringValue=[result[@"installed"][@"bootstrap_damaged"] boolValue] ? @"The permanent launcher is damaged. Upgrade the bundled installation to repair it." : [result[@"recovery_required"] boolValue] ? @"An interrupted installation needs recovery. Choose Recover / retry." : [self.state isEqualToString:@"damaged"] ? @"This installation could not be verified. Choose Recover / retry, or select a verified installation with Choose installation." : [result[@"choice_required"] boolValue] && !self.legacySource ? @"Multiple installations were found. Select the one you want to upgrade." : @"Ready. Choose clients and a workspace, then Install or Update.";
            for(NSDictionary *client in result[@"clients"]){if([client[@"error"] isKindOfClass:NSString.class])self.status.stringValue=[self.status.stringValue stringByAppendingFormat:@" %@",client[@"error"]];}
            if(missing.count)self.status.stringValue=[self.status.stringValue stringByAppendingFormat:@" %@ was not found. Install the client before selecting it.",[missing componentsJoinedByString:@" / "]];
            return;
        }
        if(result[@"preparation_id"]){self.preparation=result[@"preparation_id"];self.cancelButton.enabled=YES;self.state=@"prepared";self.installButton.title=@"Install now";self.status.stringValue=[NSString stringWithFormat:@"Ready to install into %@. The MCP connection passed. Existing settings, drawings and client preferences will be preserved. Choose Install now to activate.%@",self.installation,[result[@"skipped_skills"] count] ? @" User-owned skills will be preserved without management." : @""];return;}
        NSInteger requestedChannel=self.channel.indexOfSelectedItem;
        if(result[@"runtime_build"])[self showInstalled:result];
        if([arguments containsObject:@"--check"])[self.channel selectItemAtIndex:requestedChannel];
        NSDictionary *available=result[@"available"];if(available)self.available.stringValue=[NSString stringWithFormat:@"Available: %@\nRuntime: %@ · Instructions: %@",available[@"distribution"],available[@"runtime_build"],available[@"instructions_version"]];
        if([result[@"state"] isEqualToString:@"completed"] && [result[@"changed"] isEqual:@NO]){self.preparation=nil;self.cancelButton.enabled=NO;self.installButton.title=@"Upgrade existing installation";[self setPhase:@"completed" message:@"This installation is already up to date. No client restart is needed."];}
        else if([result[@"client_reconnect_required"] boolValue]){self.preparation=nil;self.cancelButton.enabled=NO;self.installButton.title=@"Upgrade existing installation";[self setPhase:@"completed" message:@"Completed. Restart Codex / Claude Code to load the update and its agent skill."];}
        else if([arguments containsObject:@"--check"])[self setPhase:@"installed" message:[result[@"update_available"] boolValue] ? @"An update is available. Choose Update, or an individual component." : @"No update is available for the selected channel."];
        else if([arguments containsObject:@"install-recover"]){self.preparation=nil;[self inspect];}
        else [self setPhase:@"installed" message:@"Ready. Updates run when you choose an action."];
        NSArray *skipped=result[@"skipped_skills"];if(skipped.count)self.status.stringValue=[self.status.stringValue stringByAppendingString:@" Skipped skills were preserved."];
    });
}
- (void)showInstalled:(NSDictionary *)result {
    NSString *manager=[[NSBundle mainBundle] objectForInfoDictionaryKey:@"InkscapeMCPBuildID"] ?: @"unknown";
    NSString *managerDistribution=[[NSBundle mainBundle] objectForInfoDictionaryKey:@"InkscapeMCPDistribution"] ?: @"development";
    NSDictionary *signature=result[@"manager_helper_signing"];
    NSString *signing=[signature[@"status"] isEqualToString:@"developer_id_verified"] ? [NSString stringWithFormat:@"Developer ID verified (%@)",signature[@"team_id"]] : [signature[@"status"] isEqualToString:@"ad_hoc"] ? @"Development (ad-hoc)" : @"Not verified";
    self.installed.stringValue=[NSString stringWithFormat:@"Distribution: %@ · Manager distribution: %@\nRuntime: %@\nInstructions: %@\nManager: %@\nLauncher: %@ · Compatibility: %@\nManager helper signing: %@ · Notarization: not checked on this Mac",result[@"distribution"],managerDistribution,result[@"runtime_build"],result[@"instructions_version"],manager,[result[@"launcher_build"] isKindOfClass:NSString.class] ? result[@"launcher_build"] : @"unknown (upgrade to record)",result[@"launcher_version"],signing];
    if(result[@"channel"])[self.channel selectItemAtIndex:[result[@"channel"] isEqualToString:@"stable"] ? 0 : 1];
}
@end
int main(int argc,const char **argv){@autoreleasepool{NSApplication *app=[NSApplication sharedApplication];Manager *delegate=[Manager new];app.delegate=delegate;[app setActivationPolicy:NSApplicationActivationPolicyRegular];[app run];}return 0;}
