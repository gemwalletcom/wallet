// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

@Observable
@MainActor
public final class LockWindow {
    public let lockModel: LockSceneViewModel
    public private(set) var overlayWindow: UIWindow?

    private let sceneWindow: @MainActor () -> UIWindow?
    private var userInterfaceStyle: UIUserInterfaceStyle = .unspecified

    public init(
        lockModel: LockSceneViewModel,
        sceneWindow: @escaping @MainActor () -> UIWindow?,
    ) {
        self.lockModel = lockModel
        self.sceneWindow = sceneWindow
    }

    public var showLockScreen: Bool {
        lockModel.shouldShowLockScreen
    }

    public var isPrivacyLockVisible: Bool {
        lockModel.isPrivacyLockVisible
    }

    public func setColorScheme(_ colorScheme: ColorScheme) {
        userInterfaceStyle = switch colorScheme {
        case .dark: .dark
        default: .light
        }
        overlayWindow?.overrideUserInterfaceStyle = userInterfaceStyle
    }

    public func toggleLock(show: Bool) {
        show ? presentLockWindow() : dismissLockWindow()
    }

    public func togglePrivacyLock(visible: Bool) {
        let alpha: CGFloat = visible ? 1 : 0

        if overlayWindow?.alpha != alpha {
            overlayWindow?.alpha = alpha
        }
    }
}

// MARK: - Private

extension LockWindow {
    private func presentLockWindow() {
        if overlayWindow == nil, let window = sceneWindow() {
            overlayWindow = configured(window)
        }

        if overlayWindow?.alpha != lockModel.privacyLockAlpha {
            overlayWindow?.alpha = lockModel.privacyLockAlpha
        }
        overlayWindow?.isHidden = false
    }

    private func dismissLockWindow() {
        guard !lockModel.isPrivacyLockVisible else { return }
        overlayWindow?.alpha = 0
        overlayWindow?.isHidden = true
    }

    private func configured(_ window: UIWindow) -> UIWindow {
        window.rootViewController = UIHostingController(rootView: LockScreenScene(model: lockModel))
        window.windowLevel = .alert + 1
        window.backgroundColor = .clear
        window.overrideUserInterfaceStyle = userInterfaceStyle
        window.alpha = lockModel.privacyLockAlpha
        window.makeKeyAndVisible()
        return window
    }
}
