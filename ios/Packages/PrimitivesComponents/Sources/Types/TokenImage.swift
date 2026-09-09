// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemLocalTokenIcon
import Style
import SwiftUI

public struct TokenImage: Sendable {
    private let token: GemLocalTokenIcon

    public init(token: GemLocalTokenIcon) {
        self.token = token
    }

    public var image: Image {
        switch token {
        case .usdt: Images.Tokens.usdt
        case .usdc: Images.Tokens.usdc
        }
    }
}
