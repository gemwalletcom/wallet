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
        lockModel.isCovered
    }

    public func observeScene() {
        let phases: [(Notification.Name, ScenePhase)] = [
            (UIScene.willDeactivateNotification, .inactive),
            (UIScene.didEnterBackgroundNotification, .background),
            (UIScene.willEnterForegroundNotification, .inactive),
            (UIScene.didActivateNotification, .active),
        ]
        for (name, phase) in phases {
            NotificationCenter.default.addObserver(forName: name, object: nil, queue: nil) { [weak self] _ in
                MainActor.assumeIsolated {
                    self?.onScenePhase(phase)
                }
            }
        }
        if UIApplication.shared.applicationState == .active {
            onScenePhase(.active)
        }
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
}

// MARK: - Private

extension LockWindow {
    func onScenePhase(_ phase: ScenePhase) {
        lockModel.onScenePhase(phase)
        toggleLock(show: showLockScreen)
    }

    private func presentLockWindow() {
        if overlayWindow == nil, let window = sceneWindow() {
            overlayWindow = configured(window)
        }
        overlayWindow?.isHidden = false
    }

    private func dismissLockWindow() {
        overlayWindow?.isHidden = true
    }

    private func configured(_ window: UIWindow) -> UIWindow {
        window.rootViewController = UIHostingController(rootView: LockScene(model: lockModel))
        window.windowLevel = .alert + 1
        window.backgroundColor = .clear
        window.overrideUserInterfaceStyle = userInterfaceStyle
        window.makeKeyAndVisible()
        return window
    }
}
