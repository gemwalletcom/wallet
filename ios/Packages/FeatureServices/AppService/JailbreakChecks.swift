import Foundation
import MachO
import UIKit

enum JailbreakChecks {
    @MainActor static func hasJailbreakURLScheme() -> Bool {
        ["cydia://", "sileo://", "zbra://"].contains { value in
            guard let url = URL(string: value) else { return false }
            return UIApplication.shared.canOpenURL(url)
        }
    }

    static func hasSuspiciousPaths(exists: (String) -> Bool = pathExists) -> Bool {
        suspiciousPaths.contains(where: exists)
    }

    static func pathExists(_ path: String) -> Bool {
        var metadata = stat()
        return lstat(path, &metadata) == 0
    }

    static func canCreateFileOutsideSandbox() -> Bool {
        let path = "/private/gem-jailbreak-\(UUID().uuidString)"
        let descriptor = open(path, O_WRONLY | O_CREAT | O_EXCL, S_IRUSR | S_IWUSR)
        guard descriptor >= 0 else { return false }
        defer {
            close(descriptor)
            unlink(path)
        }
        return true
    }

    private static let suspiciousPaths: [String] = [
        "/Applications/Cydia.app",
        "/Applications/Sileo.app",
        "/Applications/Zebra.app",
        "/Applications/blackra1n.app",
        "/Applications/FakeCarrier.app",
        "/Applications/Icy.app",
        "/Applications/IntelliScreen.app",
        "/Applications/MxTube.app",
        "/Applications/RockApp.app",
        "/Applications/SBSettings.app",
        "/Applications/WinterBoard.app",
        "/var/jb",
        "/Library/MobileSubstrate/DynamicLibraries/LiveClock.plist",
        "/Library/MobileSubstrate/DynamicLibraries/Veency.plist",
        "/private/var/lib/apt",
        "/private/var/lib/cydia",
        "/private/var/mobile/Library/SBSettings/Themes",
        "/private/var/stash",
        "/private/var/tmp/cydia.log",
        "/System/Library/LaunchDaemons/com.ikey.bbot.plist",
        "/System/Library/LaunchDaemons/com.saurik.Cydia.Startup.plist",
        "/usr/bin/sshd",
        "/etc/apt",
        "/Library/MobileSubstrate/MobileSubstrate.dylib",
    ]

    static func isSuspiciousLibrary(_ path: String) -> Bool {
        let name = URL(fileURLWithPath: path).lastPathComponent.lowercased()
        return ["frida", "cynject", "libcycript", "ellekit", "mobilesubstrate", "libhooker"].contains { name.contains($0) }
    }

    static func hasInjectedLibraries() -> Bool {
        for index in 0 ..< _dyld_image_count() {
            guard let pointer = _dyld_get_image_name(index), let name = String(validatingCString: pointer) else { continue }
            if isSuspiciousLibrary(name) {
                return true
            }
        }
        return false
    }

    static func hasOpenFridaPort() -> Bool {
        var serverAddress = sockaddr_in()
        serverAddress.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        serverAddress.sin_family = sa_family_t(AF_INET)
        serverAddress.sin_addr.s_addr = inet_addr("127.0.0.1")
        serverAddress.sin_port = in_port_t(27042).bigEndian
        let sock = socket(AF_INET, SOCK_STREAM, 0)
        guard sock >= 0 else { return false }
        defer { close(sock) }
        guard fcntl(sock, F_SETFL, O_NONBLOCK) == 0 else { return false }

        let result = withUnsafePointer(to: &serverAddress) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                connect(sock, $0, socklen_t(MemoryLayout<sockaddr_in>.stride))
            }
        }
        if result == 0 {
            return true
        }
        guard errno == EINPROGRESS else { return false }
        var descriptor = pollfd(fd: sock, events: Int16(POLLOUT), revents: 0)
        guard poll(&descriptor, 1, 100) > 0 else { return false }
        var error: Int32 = 0
        var length = socklen_t(MemoryLayout<Int32>.size)
        return getsockopt(sock, SOL_SOCKET, SO_ERROR, &error, &length) == 0 && error == 0
    }
}
