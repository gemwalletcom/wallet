// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import GemstoneServicesTestKit
@testable import AppLock
import SwiftUI

@MainActor
public final class LockWindowMock: LockWindowPresentable {
    public var lockModel: LockSceneViewModel
    public var overlayWindow: UIWindow?

    private var userInterfaceStyle: UIUserInterfaceStyle = .unspecified

    public init(lockModel: LockSceneViewModel) {
        self.lockModel = lockModel
    }

    public var showLockScreen: Bool {
        lockModel.shouldShowLockScreen
    }

    public var isPrivacyLockVisible: Bool {
        lockModel.isPrivacyLockVisible
    }

    public func setPhase(phase: ScenePhase) {
        guard lockModel.isAutoLockEnabled else {
            lockModel.resetLockState()
            return
        }
        lockModel.handleSceneChange(to: phase)
    }

    public func setColorScheme(_ colorScheme: ColorScheme) {
        userInterfaceStyle = switch colorScheme {
        case .dark: .dark
        default: .light
        }
        overlayWindow?.overrideUserInterfaceStyle = userInterfaceStyle
    }

    public func toggleLock(show: Bool) {
        show ? present() : dismiss()
    }

    public func togglePrivacyLock(visible: Bool) {
        let alpha: CGFloat = visible ? 1 : 0
        if overlayWindow?.alpha != alpha { overlayWindow?.alpha = alpha }
    }

    private func present() {
        if overlayWindow == nil {
            overlayWindow = Self.makeWindow(
                model: lockModel,
                visible: lockModel.isPrivacyLockVisible,
                userInterfaceStyle: userInterfaceStyle,
            )
        }
        togglePrivacyLock(visible: lockModel.isPrivacyLockVisible)
        overlayWindow?.isHidden = false
    }

    private func dismiss() {
        guard !lockModel.isPrivacyLockVisible else { return }
        overlayWindow?.alpha = 0
        overlayWindow?.isHidden = true
    }

    private static func makeWindow(
        model: LockSceneViewModel,
        visible: Bool,
        userInterfaceStyle: UIUserInterfaceStyle,
    ) -> UIWindow {
        let host = UIHostingController(rootView: LockScreenScene(model: model))
        let window = UIWindow(frame: .zero)
        window.rootViewController = host
        window.windowLevel = .alert + 1
        window.backgroundColor = .clear
        window.overrideUserInterfaceStyle = userInterfaceStyle
        window.alpha = visible ? 1 : 0
        window.makeKeyAndVisible()
        return window
    }

    public static func mock(service: any BiometryAuthenticatable = BiometryAuthenticationMock()) -> LockWindowMock {
        LockWindowMock(lockModel: LockSceneViewModel(service: service))
    }
}
