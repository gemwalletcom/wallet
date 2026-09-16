// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAppLockScreen
import SwiftUI

@Observable
@MainActor
public final class LockWindow: LockWindowPresentable {
    public let lockModel: LockSceneViewModel

    private var overlayWindow: UIWindow?
    private var userInterfaceStyle: UIUserInterfaceStyle = .unspecified

    public init(lockModel: LockSceneViewModel) {
        self.lockModel = lockModel
    }

    public var screen: GemAppLockScreen {
        lockModel.viewState.screen
    }

    public func setPhase(_ phase: ScenePhase) {
        lockModel.handleSceneChange(to: phase)
    }

    public func setColorScheme(_ colorScheme: ColorScheme) {
        userInterfaceStyle = switch colorScheme {
        case .dark: .dark
        default: .light
        }
        overlayWindow?.overrideUserInterfaceStyle = userInterfaceStyle
    }

    public func present(_ screen: GemAppLockScreen) {
        switch screen {
        case .hidden:
            overlayWindow?.isHidden = true
        case .cover, .lock:
            if overlayWindow == nil, let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene {
                overlayWindow = makeOverlayWindow(in: scene)
            }
            overlayWindow?.isHidden = false
        }
    }
}

// MARK: - Private

extension LockWindow {
    private func makeOverlayWindow(in scene: UIWindowScene) -> UIWindow {
        let window = UIWindow(windowScene: scene)
        window.rootViewController = UIHostingController(rootView: LockScreenScene(model: lockModel))
        window.windowLevel = .alert + 1
        window.backgroundColor = .clear
        window.overrideUserInterfaceStyle = userInterfaceStyle
        return window
    }
}
