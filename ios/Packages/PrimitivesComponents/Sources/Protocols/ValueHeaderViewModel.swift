// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public protocol ValueHeaderViewModel {
    var isWatchWallet: Bool { get }
    var assetImage: AssetImage? { get }
    var title: String { get }
    var subtitle: String? { get }
    var subtitleImage: Image? { get }
    var subtitleColor: Color { get }
    var buttons: [HeaderButton] { get }
}

public extension ValueHeaderViewModel {
    var subtitleImage: Image? {
        nil
    }
}
