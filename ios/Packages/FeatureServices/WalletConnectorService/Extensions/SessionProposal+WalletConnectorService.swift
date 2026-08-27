// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import WalletConnectSign

public extension Session.Proposal {
    var supportedRequiredChains: Set<Chain>? {
        requiredNamespaces.fullySupportedChains
    }

    var supportedOptionalChains: Set<Chain> {
        optionalNamespaces?.supportedChains ?? []
    }
}

private extension [String: ProposalNamespace] {
    var fullySupportedChains: Set<Chain>? {
        let blockchains = values.flatMap { $0.chains ?? [] }
        let chains = blockchains.compactMap(\.chain)
        guard chains.count == blockchains.count else { return .none }
        return chains.asSet()
    }

    var supportedChains: Set<Chain> {
        values
            .flatMap { $0.chains ?? [] }
            .compactMap(\.chain)
            .asSet()
    }
}
