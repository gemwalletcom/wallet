@testable import AppService
import Testing

struct JailbreakChecksTests {
    @Test(arguments: ["/var/jb", "/Applications/Sileo.app", "/Applications/Zebra.app", "/Applications/Cydia.app"])
    func jailbreakPaths(path: String) {
        #expect(JailbreakChecks.hasSuspiciousPaths { $0 == path })
    }

    @Test(arguments: ["/var/jb/usr/lib/ellekit/ElleKit.dylib", "/Library/MobileSubstrate/MobileSubstrate.dylib", "/usr/lib/libhooker.dylib", "/usr/lib/FridaGadget.dylib"])
    func injectedLibraries(path: String) {
        #expect(JailbreakChecks.isSuspiciousLibrary(path))
    }
}
