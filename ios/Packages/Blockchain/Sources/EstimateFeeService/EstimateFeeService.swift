// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

struct EstimateFeeService: Sendable {
    init() {}

    func provider(chain: Primitives.Chain) throws -> any GemGatewayEstimateFee {
        switch chain.type {
        case .bitcoin: try BitcoinService(chain: BitcoinChain(id: chain.rawValue))
        case .cardano: CardanoService()
        case .polkadot: PolkadotService()
        default: EmptyService()
        }
    }

    func getFee(chain: Gemstone.Chain, input: Gemstone.GemTransactionLoadInput) async throws -> Gemstone.GemTransactionLoadFee? {
        try await provider(chain: chain.map()).getFee(chain: chain, input: input)
    }

    func getFeeData(chain: Gemstone.Chain, input: Gemstone.GemTransactionLoadInput) async throws -> String? {
        try await provider(chain: chain.map()).getFeeData(chain: chain, input: input)
    }
}

final class EmptyService: Sendable {
    init() {}
}

extension EmptyService: GemGatewayEstimateFee {
    func getFee(chain _: Gemstone.Chain, input _: Gemstone.GemTransactionLoadInput) async throws -> Gemstone.GemTransactionLoadFee? {
        .none
    }

    func getFeeData(chain _: Gemstone.Chain, input _: GemTransactionLoadInput) async throws -> String? {
        .none
    }
}
