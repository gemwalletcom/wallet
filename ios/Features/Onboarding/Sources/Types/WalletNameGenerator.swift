// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import GemstoneServices

struct WalletNameGenerator {
    private let type: ImportWalletType
    private let walletService: WalletService

    init(type: ImportWalletType, walletService: WalletService) {
        self.type = type
        self.walletService = walletService
    }

    func name() async -> String {
        name(
            type: type,
            index: (try? await walletService.nextWalletIndex()) ?? .zero,
        )
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
