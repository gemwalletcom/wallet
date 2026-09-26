// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import WalletConnector

public extension ConnectionProposalSceneViewModel {
    static func mock(metadata: ApplicationMetadata = .mock()) -> ConnectionProposalSceneViewModel {
        ConnectionProposalSceneViewModel(
            confirmTransferDelegate: { _ in },
            pairingProposal: .mock(proposal: .mock(metadata: metadata)),
        )
    }
}
