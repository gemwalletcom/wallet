// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import protocol Gemstone.GemWalletServiceProtocol
import GemstonePrimitives

struct WalletNameGenerator {
    private let type: ImportWalletType
    private let wallets: [Wallet]
    private let service: any GemWalletServiceProtocol

    init(type: ImportWalletType, wallets: [Wallet], service: any GemWalletServiceProtocol) {
        self.type = type
        self.wallets = wallets
        self.service = service
    }

    func name() -> String {
        name(type: type, index: service.nextWalletIndex(wallets: wallets))
    }

    private func name(type: ImportWalletType, index: Int) -> String {
        switch type {
        case .multicoin: Localized.Wallet.defaultName(index)
        case let .chain(chain): Localized.Wallet.defaultNameChain(chain.networkName, index)
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
