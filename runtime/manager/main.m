#import <Cocoa/Cocoa.h>

// Thin UI: all compatibility, merging, download and recovery stay in the Rust launcher.
@interface Manager : NSObject <NSApplicationDelegate>
@property NSWindow *window;
@property NSTextField *installed;
@property NSTextField *available;
@property NSTextField *status;
@property NSPopUpButton *channel;
@property NSProgressIndicator *progress;
@property NSMutableArray<NSButton *> *buttons;
@property NSString *installation;
@property NSTask *running;
@end
@implementation Manager
- (NSButton *)button:(NSString *)title action:(SEL)action {
    NSButton *button=[NSButton buttonWithTitle:title target:self action:action];
    [self.buttons addObject:button]; return button;
}
- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    self.installation=[[NSUserDefaults standardUserDefaults] stringForKey:@"InstallationDirectory"] ?: [NSHomeDirectory() stringByAppendingPathComponent:@"Library/Application Support/inkscape-mcp"];
    self.buttons=[NSMutableArray new];
    self.window=[[NSWindow alloc] initWithContentRect:NSMakeRect(0,0,620,400) styleMask:NSWindowStyleMaskTitled|NSWindowStyleMaskClosable|NSWindowStyleMaskMiniaturizable backing:NSBackingStoreBuffered defer:NO];
    self.window.title=@"Inkscape MCP Manager";
    NSStackView *stack=[NSStackView new]; stack.orientation=NSUserInterfaceLayoutOrientationVertical;stack.alignment=NSLayoutAttributeLeading;stack.spacing=16;stack.edgeInsets=NSEdgeInsetsMake(24,24,24,24);
    self.installed=[NSTextField wrappingLabelWithString:@"Reading installed versions…"];
    self.available=[NSTextField wrappingLabelWithString:@"Check for available updates when ready."];
    self.status=[NSTextField wrappingLabelWithString:@"Opening this window preserves open drawings and client sessions."];
    self.channel=[NSPopUpButton new];[self.channel addItemsWithTitles:@[@"stable",@"prerelease"]];
    [stack addArrangedSubview:self.installed];[stack addArrangedSubview:self.available];
    NSStackView *channelRow=[NSStackView stackViewWithViews:@[[NSTextField labelWithString:@"Update channel"],self.channel]];channelRow.spacing=12;[stack addArrangedSubview:channelRow];
    NSStackView *row=[NSStackView stackViewWithViews:@[[self button:@"Check updates" action:@selector(check:)],[self button:@"Instructions" action:@selector(instructions:)],[self button:@"Runtime" action:@selector(runtime:)],[self button:@"Update both" action:@selector(both:)]]];row.spacing=8;[stack addArrangedSubview:row];
    NSStackView *recovery=[NSStackView stackViewWithViews:@[[self button:@"Roll back" action:@selector(rollback:)],[self button:@"Choose installation…" action:@selector(choose:)]]];recovery.spacing=8;[stack addArrangedSubview:recovery];
    self.progress=[NSProgressIndicator new];self.progress.style=NSProgressIndicatorStyleBar;self.progress.indeterminate=YES;[stack addArrangedSubview:self.progress];[stack addArrangedSubview:self.status];
    stack.translatesAutoresizingMaskIntoConstraints=NO;[self.window.contentView addSubview:stack];
    [NSLayoutConstraint activateConstraints:@[[stack.leadingAnchor constraintEqualToAnchor:self.window.contentView.leadingAnchor],[stack.trailingAnchor constraintEqualToAnchor:self.window.contentView.trailingAnchor],[stack.topAnchor constraintEqualToAnchor:self.window.contentView.topAnchor],[self.progress.widthAnchor constraintEqualToConstant:540]]];
    [self.window center];[self.window makeKeyAndOrderFront:nil];[NSApp activateIgnoringOtherApps:YES];[self run:@[@"--version"]];
}
- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {return YES;}
- (NSApplicationTerminateReply)applicationShouldTerminate:(NSApplication *)sender {if(self.running){self.status.stringValue=@"Wait for the current operation to finish before closing.";return NSTerminateCancel;}return NSTerminateNow;}
- (void)choose:(id)sender {
    NSOpenPanel *panel=[NSOpenPanel openPanel];panel.canChooseDirectories=YES;panel.canChooseFiles=NO;panel.allowsMultipleSelection=NO;panel.message=@"Choose the permanent inkscape-mcp installation directory.";
    if([panel runModal]==NSModalResponseOK){self.installation=panel.URL.path;[[NSUserDefaults standardUserDefaults] setObject:self.installation forKey:@"InstallationDirectory"];[self run:@[@"--version"]];}
}
- (void)check:(id)sender {[self update:@[@"--check"]];}
- (void)instructions:(id)sender {[self update:@[@"--instructions"]];}
- (void)runtime:(id)sender {[self update:@[@"--runtime"]];}
- (void)both:(id)sender {[self update:@[]];}
- (void)rollback:(id)sender {[self run:@[@"rollback"]];}
- (void)update:(NSArray *)options {NSMutableArray *args=[NSMutableArray arrayWithObject:@"update"];[args addObjectsFromArray:options];[args addObjectsFromArray:@[@"--channel",self.channel.titleOfSelectedItem]];[self run:args];}
static NSData *boundedRead(NSFileHandle *file,NSTask *task) {
    NSMutableData *data=[NSMutableData new];
    while(YES){NSData *part=[file readDataOfLength:4096];if(!part.length)break;if(data.length+part.length>1024*1024){if(task.running)[task terminate];break;}[data appendData:part];}
    return data;
}
- (void)run:(NSArray *)arguments {
    if(self.running)return;
    NSString *executable=[self.installation stringByAppendingPathComponent:@"bin/inkscape-mcp-launcher"];
    if(![[NSFileManager defaultManager] isExecutableFileAtPath:executable]){self.status.stringValue=@"No launcher found. Run setup with --independent-updates once, or choose an existing installation.";return;}
    NSTask *task=[NSTask new];task.executableURL=[NSURL fileURLWithPath:executable];NSMutableArray *args=[arguments mutableCopy];[args addObjectsFromArray:@[@"--install-dir",self.installation,@"--json"]];task.arguments=args;
    task.standardInput=[NSFileHandle fileHandleWithNullDevice];NSPipe *output=[NSPipe pipe];NSPipe *errors=[NSPipe pipe];task.standardOutput=output;task.standardError=errors;
    NSError *error=nil;if(![task launchAndReturnError:&error]){self.status.stringValue=error.localizedDescription;return;}
    self.running=task;for(NSButton *button in self.buttons)button.enabled=NO;self.channel.enabled=NO;[self.progress startAnimation:nil];self.status.stringValue=@"Working… Downloading, validating and preparing changes may take a few minutes.";
    dispatch_group_t group=dispatch_group_create();__block NSData *out=nil;__block NSData *err=nil;
    dispatch_group_async(group,dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0),^{out=boundedRead(output.fileHandleForReading,task);});
    dispatch_group_async(group,dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0),^{err=boundedRead(errors.fileHandleForReading,task);});
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW,600*NSEC_PER_SEC),dispatch_get_main_queue(),^{if(self.running==task && task.running)[task terminate];});
    dispatch_group_async(group,dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0),^{[task waitUntilExit];});
    dispatch_group_notify(group,dispatch_get_main_queue(),^{
        self.running=nil;for(NSButton *button in self.buttons)button.enabled=YES;self.channel.enabled=YES;[self.progress stopAnimation:nil];
        NSDictionary *result=out.length ? [NSJSONSerialization JSONObjectWithData:out options:0 error:nil] : nil;
        if(![result isKindOfClass:NSDictionary.class]){self.status.stringValue=err.length ? [[NSString alloc] initWithData:err encoding:NSUTF8StringEncoding] : @"Operation interrupted. The launcher will recover before the next launch or update.";return;}
        if(task.terminationStatus!=0){self.status.stringValue=result[@"error"] ?: @"Update failed. Existing installation is preserved.";return;}
        if(result[@"runtime_build"]){self.installed.stringValue=[NSString stringWithFormat:@"Installed runtime: %@\nRevision: %@\nInstructions: %@\nLauncher: %@",result[@"runtime_build"],result[@"runtime_revision"],result[@"instructions_version"],result[@"launcher_version"]];if([arguments containsObject:@"--version"] && result[@"channel"])[self.channel selectItemWithTitle:result[@"channel"]];}
        NSDictionary *available=result[@"available"];if(available){self.available.stringValue=[NSString stringWithFormat:@"Available: %@\nRuntime: %@ · Instructions: %@",available[@"distribution"],available[@"runtime_build"],available[@"instructions_version"]];}
        if([result[@"client_reconnect_required"] boolValue])self.status.stringValue=[result[@"skill_reload_required"] boolValue] ? @"Activated. Reconnect your MCP client and reload its skill. Open Inkscape drawings remain available." : @"Activated. Reconnect your MCP client. Open Inkscape drawings remain available.";
        else if([arguments containsObject:@"--check"])self.status.stringValue=[result[@"update_available"] boolValue] ? @"An update is available. Choose instructions, runtime, or both." : @"Your selected components are up to date.";
        else self.status.stringValue=@"Ready. Updates run only when you choose an action.";
    });
}
@end
int main(int argc,const char **argv){@autoreleasepool{NSApplication *app=[NSApplication sharedApplication];Manager *delegate=[Manager new];app.delegate=delegate;[app setActivationPolicy:NSApplicationActivationPolicyRegular];[app run];}return 0;}
