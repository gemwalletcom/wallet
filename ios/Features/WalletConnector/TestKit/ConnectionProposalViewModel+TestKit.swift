// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import WalletConnector

public extension ConnectionProposalViewModel {
    static func mock(metadata: ApplicationMetadata = .mock()) -> ConnectionProposalViewModel {
        ConnectionProposalViewModel(
            confirmTransferDelegate: { _ in },
            pairingProposal: .mock(proposal: .mock(metadata: metadata)),
        )
    }
}
