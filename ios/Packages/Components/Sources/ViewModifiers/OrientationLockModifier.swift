// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import UIKit

@MainActor
public enum OrientationLock {
    public static var mask: UIInterfaceOrientationMask = .all
}

private struct OrientationLockModifier: ViewModifier {
    let mask: UIInterfaceOrientationMask

    func body(content: Content) -> some View {
        content
            .onAppear { update(mask) }
            .onDisappear { update(.all) }
    }

    private func update(_ mask: UIInterfaceOrientationMask) {
        OrientationLock.mask = mask
        guard let scene = UIApplication.shared.connectedScenes.compactMap({ $0 as? UIWindowScene }).first else { return }
        scene.requestGeometryUpdate(.iOS(interfaceOrientations: mask))
        scene.keyWindow?.rootViewController?.setNeedsUpdateOfSupportedInterfaceOrientations()
    }
}

public extension View {
    func orientationLock(_ mask: UIInterfaceOrientationMask) -> some View {
        modifier(OrientationLockModifier(mask: mask))
    }
}
