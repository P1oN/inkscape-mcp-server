// Hidden AppKit state/routing regression fixture. Never launches a helper or client.
#define main manager_application_main
#import "main.m"
#undef main

@interface TestManager : Manager
@property NSArray *lastArguments;
@property NSUInteger checksStarted;
@property NSDictionary *activatedReview;
@end
@implementation TestManager
- (void)run:(NSArray *)arguments request:(NSDictionary *)request {self.lastArguments=arguments;}
- (void)startUpdateCheck {self.checksStarted++;}
- (void)beginManagerActivation:(NSDictionary *)result {self.activatedReview=result;}
@end
static void require(BOOL condition, NSString *message) {
    if(!condition){fprintf(stderr,"%s\n",message.UTF8String);exit(1);}
}
int main(void) {@autoreleasepool {
    [NSApplication sharedApplication];[NSApp setActivationPolicy:NSApplicationActivationPolicyProhibited];
    TestManager *manager=[TestManager new];[manager buildWindow];
    for(NSString *json in @[@"[]",@"1",@"\"text\"",@"null",@"{broken"]){require(managerResponse([json dataUsingEncoding:NSUTF8StringEncoding])==nil,@"malformed helper root reaches dictionary subscripting");}
    [manager applyInspection:@{@"state":@"not_installed",@"clients":@[],@"candidates":@[]}];
    require(manager.codex.state==NSControlStateValueOn && manager.skills.state==NSControlStateValueOn,@"default registration changed");
    require(!manager.installButton.enabled && !manager.checkButton.hidden && manager.checkButton.enabled && !manager.rollbackButton.enabled,@"fresh install cannot check bundled updates");
    require(manager.checksStarted==1,@"first inspection does not check updates");
    [manager applyInspection:@{@"state":@"not_installed",@"clients":@[@{@"name":@"codex",@"error":NSNull.null,@"profile":@"/owned/custom-codex",@"config":@"/owned/custom-codex/config.toml",@"binding":@{@"explanation":@"Confirmed stale source; review recovery."}}],@"candidates":@[]}];
    require([manager.clientHint.stringValue containsString:@"Confirmed stale source"] && [manager.clientHint.stringValue containsString:@"/owned/custom-codex/config.toml"],@"JSON null suppresses ownership guidance or effective profile");
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
    require([manager.installButton.title isEqual:@"Review update"] && [manager.lastArguments.firstObject isEqual:@"update"] && [manager.lastArguments containsObject:@"--check"],@"available update bypasses component review");
    NSDictionary *available=@{@"update_available":@YES,@"available":@{@"distribution":@"v99.0.0"}};
    NSUInteger generation=manager.generation;NSInteger channel=manager.channel.indexOfSelectedItem;NSString *installation=manager.installation;
    [manager applyCheck:available error:nil generation:generation installation:installation channel:channel];require(manager.updateAvailable && [manager.available.stringValue containsString:@"v99.0.0"],@"new update not offered");
    [manager applyCheck:@{@"update_available":@NO} error:nil generation:generation installation:installation channel:channel];require(!manager.updateAvailable,@"no-update response retained old offer");
    for(NSString *error in @[@"Offline",@"GitHub failed",@"The request timed out."]){[manager applyCheck:nil error:error generation:generation installation:installation channel:channel];require(!manager.updateAvailable && [manager.available.stringValue containsString:@"Local setup remains available"],@"network error blocks local setup");}
    [manager changeOptions:manager.codex];NSString *screen=manager.available.stringValue;
    [manager applyCheck:available error:nil generation:generation installation:installation channel:channel];require([manager.available.stringValue isEqual:screen] && !manager.updateAvailable,@"stale response replaced changed choices");
    [manager applyCheck:available error:nil generation:manager.generation installation:@"/other" channel:channel];require(!manager.updateAvailable,@"other installation response accepted");
    manager.preparation=@"reviewed";[manager applyCheck:available error:nil generation:manager.generation installation:installation channel:channel];require(!manager.updateAvailable,@"check replaced a prepared review");manager.preparation=nil;
    manager.running=[NSTask new];[manager applyCheck:available error:nil generation:manager.generation installation:installation channel:channel];require(!manager.updateAvailable,@"check replaced activation");manager.running=nil;
    [manager.channel selectItemAtIndex:0];[manager changeOptions:manager.channel];require(!manager.updateAvailable,@"channel change keeps a stale update selection");
    manager.configurationChanged=NO;[manager.channel selectItemAtIndex:1];[manager changeOptions:manager.channel];
    [manager applyCheck:available error:nil generation:manager.generation installation:manager.installation channel:1];[manager install:nil];
    require([manager.installButton.title isEqual:@"Review update"] && [manager.lastArguments.firstObject isEqual:@"update"] && [manager.lastArguments containsObject:@"--check"],@"channel update offer routes into offline installation");
    [manager applyInspection:@{@"state":@"damaged",@"recovery_required":@YES,@"clients":@[],@"candidates":@[]}];
    require(manager.checkButton.enabled && !manager.rollbackButton.enabled && !manager.recoverButton.hidden,@"interrupted installation hides recovery or independent Manager check");
    NSDictionary *managerUpdate=@{@"manager_update_available":@YES,@"manager_available":@{@"tag":@"v100.0.0",@"manager_build":@"manager-new",@"bootstrap":@{@"build_id":@"bootstrap-new"}}};
    [manager applyManagerCheck:managerUpdate error:nil generation:manager.generation installation:manager.installation channel:manager.channel.indexOfSelectedItem];
    require(manager.managerOffer!=nil && manager.managerUpdateButton.enabled,@"incomplete/damaged runtime hides independent Manager offer");
    [manager applyCheck:nil error:@"launcher upgrade required" generation:manager.generation installation:manager.installation channel:manager.channel.indexOfSelectedItem];
    require(manager.managerOffer!=nil && manager.managerUpdateButton.enabled,@"runtime compatibility failure hides Manager update");
    manager.recoveryRequired=NO;
    [manager applyCheck:nil error:@"launcher upgrade required" generation:manager.generation installation:manager.installation channel:manager.channel.indexOfSelectedItem];
    require(manager.managerOffer!=nil && manager.managerUpdateButton.enabled && [manager.available.stringValue containsString:@"launcher upgrade required"],@"accepted runtime compatibility failure hides Manager update");
    [manager updateManager:nil];require([manager.lastArguments.firstObject isEqual:@"manager-online-prepare"],@"Manager action bypasses verified staging");
    NSUInteger oldGeneration=manager.generation;[manager invalidateCheck];
    [manager applyManagerCheck:managerUpdate error:nil generation:oldGeneration installation:manager.installation channel:manager.channel.indexOfSelectedItem];require(manager.managerOffer==nil,@"stale Manager offer accepted after review/context invalidation");
    for(NSString *error in @[@"offline",@"The request timed out."]){[manager applyManagerCheck:nil error:error generation:manager.generation installation:manager.installation channel:manager.channel.indexOfSelectedItem];require(manager.managerOffer==nil && !manager.managerUpdateButton.enabled,@"failed Manager check keeps stale action");}
    manager.preparation=@"review";[manager applyManagerCheck:managerUpdate error:nil generation:manager.generation installation:manager.installation channel:manager.channel.indexOfSelectedItem];require(manager.managerOffer==nil,@"Manager reply replaces prepared review");manager.preparation=nil;
    [manager applyManagerCheck:managerUpdate error:nil generation:manager.generation installation:@"/different-root" channel:manager.channel.indexOfSelectedItem];require(manager.managerOffer==nil,@"Manager reply crosses installation");
    [manager applyManagerCheck:managerUpdate error:nil generation:manager.generation installation:manager.installation channel:1-manager.channel.indexOfSelectedItem];require(manager.managerOffer==nil,@"Manager reply crosses channel");
    manager.managerOffer=nil;[manager showSettings:nil];
    require([manager.pageManagerAction.title isEqual:@"Check Manager updates"],@"settings lacks read-only check action");
    [manager applyManagerCheck:managerUpdate error:nil generation:manager.generation installation:manager.installation channel:manager.channel.indexOfSelectedItem];
    require([manager.pageManagerAction.title isEqual:@"Review Manager update"] && manager.pageManagerAction.enabled,@"accepted discovery leaves stale visible action");
    [manager applyManagerCheck:@{@"manager_update_available":@NO} error:nil generation:manager.generation installation:manager.installation channel:manager.channel.indexOfSelectedItem];
    require([manager.pageManagerAction.title isEqual:@"Check Manager updates"],@"no-update response leaves misleading Review action");
    // Cancel restores a usable offer immediately, but never retains activation consent.
    NSDictionary *review=@{@"review_id":@"reviewed-hash",@"manager_application":@"/owned/Applications/Manager.app",@"baseline_healthy":@YES,@"installed_manager":@{@"InkscapeMCPDistribution":@"v1.0.0",@"InkscapeMCPBuildID":@"manager-old",@"InkscapeMCPRevision":@"old-revision"},@"target_manager":@{@"tag":@"v2.0.0",@"manager_build":@"manager-new",@"manager_revision":@"new-revision",@"publisher":@"publisher",@"bootstrap":@{@"build_id":@"bootstrap-new",@"source_revision":@"new-source",@"executable":@{@"sha256":@"full-target-hash"}},@"launcher_version":@2,@"text_interface":@1,@"helper_protocol":@5,@"component_format":@1},@"bootstrap_changes":@[@{@"destination":@"/owned/custom profile/bin/inkscape-mcp",@"from":@{@"sha256":@"full-before-hash"},@"to":@{@"sha256":@"full-after-hash"}}]};
    manager.reviewOffer=managerUpdate[@"manager_available"];[manager invalidateCheck];[manager presentManagerReview:review];
    require([manager.managerPage isEqual:@"review"] && manager.pagePrimary.enabled && manager.connectionScroll.hidden,@"review is not a separate page");
    require([[manager reviewDetails] containsString:@"full-before-hash"] && [[manager reviewDetails] containsString:@"full-after-hash"],@"exact changes lose full identities");
    [manager applyManagerCheck:managerUpdate error:nil generation:manager.generation installation:manager.installation channel:manager.channel.indexOfSelectedItem];
    require(manager.managerOffer==nil && manager.managerReview==review,@"background check replaces consent context");
    [manager cancelManagerReview:nil];
    require([manager.managerPage isEqual:@"settings"] && manager.managerReview==nil && manager.managerOffer!=nil && manager.pagePrimary.enabled && manager.managerUpdateButton.enabled && manager.activatedReview==nil,@"Cancel leaves stale offer or activation consent");
    [manager updateManager:nil];require([manager.lastArguments.firstObject isEqual:@"manager-online-prepare"],@"retry after Cancel skips preparation");
    [manager presentManagerReview:review];[manager confirmManagerReview:nil];
    require(manager.activatedReview==review && manager.managerReview==nil,@"confirmation is not bound to the displayed plan");
    manager.activatedReview=nil;[manager presentManagerReview:review];[manager changeOptions:manager.channel];[manager confirmManagerReview:nil];
    require(manager.activatedReview==nil,@"context change keeps stale consent");
    [manager startupManagerFailed:@"Actual startup acknowledgment failed"];require([manager.lastArguments.firstObject isEqual:@"install-inspect"],@"startup failure does not inspect recovery state");
    [manager applyInspection:@{@"state":@"damaged",@"recovery_required":@YES,@"manager_recovery_required":@YES,@"manager_update_error":@"Owned Manager recovery details",@"clients":@[],@"candidates":@[]}];
    require([manager.managerPage isEqual:@"recovery"] && [manager.pagePrimary.title isEqual:@"Recover Manager"],@"Manager recovery is hidden in a generic footer");
    [manager applyInspection:@{@"state":@"installed",@"installed":installed,@"manager_update_recovered":@YES,@"clients":@[],@"candidates":@[]}];
    require([manager.managerOutcome isEqual:@"restored"] && [manager.managerPage isEqual:@"settings"],@"verified restoration loses its outcome");
    manager.rollbackAvailable=YES;manager.recoveryRequired=NO;[manager rollbackFromSettings:nil];
    require(manager.managerPage==nil && [manager.lastArguments.firstObject isEqual:@"rollback"],@"rollback result or failure would be hidden behind Settings");
    NSMutableDictionary *newVersions=[installed mutableCopy];newVersions[@"runtime_build"]=@"runtime-after-rollback";[manager showInstalled:newVersions];[manager showSettings:nil];
    require([manager.installedComponents[@"runtime_build"] isEqual:@"runtime-after-rollback"],@"Settings keeps an old snapshot after component operations");
    [manager startupManagerFailed:@"Original startup failure"];[manager startupInspectionFailed:@"Inspection timed out"];
    require([manager.managerPage isEqual:@"error"] && [manager.managerFailure containsString:@"Original startup failure"] && [manager.managerFailure containsString:@"Inspection timed out"],@"inspection timeout hides the original startup failure");
    manager.lastArguments=@[@"navigation-marker"];[manager backToSettings:nil];
    require([manager.managerPage isEqual:@"settings"] && [manager.lastArguments.firstObject isEqual:@"navigation-marker"],@"Back to settings retries a persistently failing inspection");
    [manager backToConnection:nil];require(manager.managerPage==nil,@"inspection failure traps navigation away from Advanced");
    [manager failManagerUpdate:@"Persistent inspection failure"];[manager retryManagerInspection:nil];require([manager.lastArguments.firstObject isEqual:@"install-inspect"],@"explicit inspection retry is not separate from navigation");
    manager.managerOutcome=@"updated";[manager applyInspection:@{@"state":@"installed",@"installed":installed,@"manager_update_recovered":@NO,@"clients":@[],@"candidates":@[]}];
    require([manager.managerOutcome isEqual:@"updated"],@"superseded recovery receipt replaces successful update outcome");
    NSString *originalRoot=manager.installation;manager.installation=@"/owned/another-installation";manager.managerWarning=@"Old root warning";
    [manager applyInspection:@{@"state":@"not_installed",@"clients":@[],@"candidates":@[]}];
    require(!manager.managerOutcome && !manager.managerWarning && !manager.managerPage,@"completion crosses installation roots");
    manager.installation=originalRoot;[manager applyInspection:@{@"state":@"installed",@"installed":installed,@"clients":@[],@"candidates":@[]}];
    [manager failManagerUpdate:@"A long permission failure remains available in the scrollable content instead of being truncated in the footer."];
    require([manager.managerPage isEqual:@"error"] && [manager.pagePrimary.title isEqual:@"Back to settings"],@"permission failure offers generic destructive retry");
    for(NSString *page in @[@"preparing",@"handoff",@"review",@"recovery",@"error",@"settings"]){
        manager.managerReview=[page isEqual:@"review"] ? review : nil;
        [manager showManagerPage:page];[manager.window setContentSize:NSMakeSize(620,600)];[manager.window.contentView layoutSubtreeIfNeeded];
        require(NSContainsRect(manager.window.contentView.bounds,[manager.pagePrimary convertRect:manager.pagePrimary.bounds toView:manager.window.contentView]),@"update primary action leaves minimum window");
        require(manager.pagePrimary.enabled==![@[@"preparing",@"handoff"] containsObject:page],@"preparing or handoff exposes an active confirmation");
    }
    manager.managerReview=nil;[manager backToConnection:nil];
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
