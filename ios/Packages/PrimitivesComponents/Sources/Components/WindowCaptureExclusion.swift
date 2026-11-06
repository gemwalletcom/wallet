// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

#if targetEnvironment(macCatalyst)
    extension View {
        func excludeFromWindowCapture() -> some View {
            onAppear { WindowCaptureExclusion.begin() }
                .onDisappear { WindowCaptureExclusion.end() }
        }
    }

    @MainActor
    private enum WindowCaptureExclusion {
        private static let sharingTypeNone = 0
        private static let sharingTypeReadOnly = 1
        private static var protectedScreens = 0

        static func begin() {
            protectedScreens += 1
            if protectedScreens == 1 {
                setSharingType(sharingTypeNone)
            }
        }

        static func end() {
            protectedScreens = max(protectedScreens - 1, 0)
            if protectedScreens == 0 {
                setSharingType(sharingTypeReadOnly)
            }
        }

        private static func setSharingType(_ sharingType: Int) {
            guard let applicationClass: AnyObject = NSClassFromString("NSApplication"),
                  let application = applicationClass.value(forKey: "sharedApplication") as? NSObject,
                  let windows = application.value(forKey: "windows") as? [NSObject]
            else {
                return
            }
            windows.forEach { $0.setValue(sharingType, forKey: "sharingType") }
        }
    }
#else
    extension View {
        func excludeFromWindowCapture() -> some View {
            self
        }
    }
#endif
