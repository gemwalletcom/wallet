// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletDefaultName
import Localization
import Primitives
import GemstonePrimitives

struct WalletNameGenerator {
    private let defaultName: GemWalletDefaultName

    init(defaultName: GemWalletDefaultName) {
        self.defaultName = defaultName
    }

    var hasExistingWallets: Bool {
        index > 1
    }

    func name() -> String {
        switch defaultName {
        case let .multicoin(index): Localized.Wallet.defaultName(Int(index))
        case let .chain(chain, index): Localized.Wallet.defaultNameChain(Chain(core: chain).networkName, Int(index))
        }
    }

    private var index: Int32 {
        switch defaultName {
        case let .multicoin(index): index
        case let .chain(_, index): index
        }
    }
}

extension ImportWalletType {
    var type: WalletType {
        switch self {
        case .multicoin: .multicoin
        case .chain: .single
        }
    }
}
