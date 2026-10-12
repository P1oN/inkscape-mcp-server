// Hidden AppKit state/routing regression fixture. Never launches a helper or client.
#define main manager_application_main
#import "main.m"
#undef main

@interface TestManager : Manager
@property NSArray *lastArguments;
@end
@implementation TestManager
- (void)run:(NSArray *)arguments request:(NSDictionary *)request {self.lastArguments=arguments;}
@end
static void require(BOOL condition, NSString *message) {
    if(!condition){fprintf(stderr,"%s\n",message.UTF8String);exit(1);}
}
int main(void) {@autoreleasepool {
    [NSApplication sharedApplication];[NSApp setActivationPolicy:NSApplicationActivationPolicyProhibited];
    TestManager *manager=[TestManager new];[manager buildWindow];
    [manager applyInspection:@{@"state":@"not_installed",@"clients":@[],@"candidates":@[]}];
    require(manager.codex.state==NSControlStateValueOn && manager.skills.state==NSControlStateValueOn,@"default registration changed");
    require(!manager.installButton.enabled && manager.checkButton.hidden && !manager.rollbackButton.enabled,@"fresh install exposes unavailable actions");
    require(manager.advancedCard.hidden && manager.recoverButton.hidden && manager.cancelButton.hidden,@"fresh setup starts cluttered");
    NSDictionary *installed=@{@"distribution":@"v-test",@"runtime_build":@"runtime-test",@"instructions_version":@"text-test",@"launcher_build":@"launcher-test",@"launcher_version":@1,@"rollback_available":@YES,@"channel":@"prerelease"};
    [manager applyInspection:@{@"state":@"installed",@"installed":installed,@"workspace":@"/owned/drawings",@"inkscape":@"/Applications/Inkscape.app/Contents/MacOS/inkscape",@"candidates":@[],@"clients":@[@{@"name":@"claude",@"available":@NO}]}];
    require(manager.installButton.enabled && manager.checkButton.enabled && manager.rollbackButton.enabled,@"installed management actions unavailable");
    require(![manager.clientHint.stringValue containsString:@"was not found"],@"unselected client masks the next step");
    NSString *versions=manager.installed.stringValue;NSString *summary=manager.summary.stringValue;
    [manager setPhase:@"failed" message:@"Owned synthetic failure"];
    require([manager.installed.stringValue isEqual:versions] && [manager.summary.stringValue isEqual:summary] && !manager.recoverButton.hidden,@"failure destroys installation context or hides recovery");
    manager.running=[NSTask new];[manager setPhase:@"busy" message:@"Owned synthetic operation"];
    require(!manager.installButton.enabled && !manager.checkButton.enabled && !manager.rollbackButton.enabled && !manager.codex.enabled && !manager.progress.hidden,@"busy operation allows conflicting actions");
    manager.running=nil;manager.preparation=@"owned-preparation";[manager setPhase:@"prepared" message:@"Reviewed changes"];
    require([manager.installButton.title isEqual:@"Install now"] && !manager.cancelButton.hidden,@"prepared review cannot be activated or cancelled");
    [manager install:nil];require([manager.lastArguments isEqual:@[@"install-activate",@"--preparation",@"owned-preparation"]],@"primary action bypasses prepared activation");
    [manager changeOptions:manager.codex];require(manager.preparation==nil && manager.cancelButton.hidden,@"changed options retain stale preparation");
    manager.updateAvailable=YES;manager.configurationChanged=NO;[manager refreshControls];[manager install:nil];
    require([manager.installButton.title isEqual:@"Update now"] && [manager.lastArguments.firstObject isEqual:@"update"],@"available update uses install flow");
    [manager.channel selectItemAtIndex:0];[manager changeOptions:manager.channel];require(!manager.updateAvailable,@"channel change keeps a stale update selection");
    [manager applyInspection:@{@"state":@"damaged",@"recovery_required":@YES,@"clients":@[],@"candidates":@[]}];
    require(!manager.checkButton.enabled && !manager.rollbackButton.enabled && !manager.recoverButton.hidden,@"interrupted installation hides recovery");
    manager.advancedButton.state=NSControlStateValueOn;[manager toggleAdvanced:nil];
    [manager setPath:@"/owned/a deliberately long workspace directory with spaces/another long directory/drawings" label:manager.workspaceLabel];
    [manager.window setContentSize:NSMakeSize(620,600)];
    for(NSAppearanceName appearance in @[NSAppearanceNameAqua,NSAppearanceNameDarkAqua]){
        manager.window.appearance=[NSAppearance appearanceNamed:appearance];
        NSView *content=manager.window.contentView;[content layoutSubtreeIfNeeded];
        require(NSContainsRect(content.bounds,[manager.installButton convertRect:manager.installButton.bounds toView:content]),@"primary action leaves the smallest window");
        require(NSContainsRect(content.bounds,[manager.status convertRect:manager.status.bounds toView:content]),@"status leaves the smallest window");
        NSRect path=[manager.workspaceLabel convertRect:manager.workspaceLabel.bounds toView:content];
        require(NSMinX(path)>=0 && NSMaxX(path)<=NSWidth(content.bounds),@"long path overflows the window");
    }
    [manager.window orderOut:nil];fprintf(stdout,"Manager UI state and action routing passed\n");
    return 0;
}}
