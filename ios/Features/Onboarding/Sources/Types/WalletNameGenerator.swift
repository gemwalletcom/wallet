// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import protocol Gemstone.GemOnboardingServiceProtocol
import GemstonePrimitives

struct WalletNameGenerator {
    private let type: ImportWalletType
    private let service: any GemOnboardingServiceProtocol

    init(type: ImportWalletType, service: any GemOnboardingServiceProtocol) {
        self.type = type
        self.service = service
    }

    func name() async -> String {
        name(
            type: type,
            index: (try? service.nextWalletIndex()) ?? .zero,
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
