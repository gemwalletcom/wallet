// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing
@testable import WalletConnector

struct ConnectionProposalViewModelTests {
    @Test
    func appIconPrefersTheProposedIcon() {
        let metadata = ApplicationMetadata.mock(url: "https://tronscan.org/some/page", icon: "https://tronscan.org/static/media/logo.png")
        let model = ConnectionProposalViewModel(
            confirmTransferDelegate: { _ in },
            pairingProposal: .mock(proposal: .mock(metadata: metadata)),
        )

        #expect(model.imageUrl?.absoluteString == "https://assets.gemwallet.com/proxy/icon?url=https%3A%2F%2Ftronscan.org%2Fstatic%2Fmedia%2Flogo.png&size=256")
        #expect(model.websiteText == "tronscan.org")
    }

    @Test
    func unsafeAppWebsiteUsesPlaceholder() {
        let model = ConnectionProposalViewModel(
            confirmTransferDelegate: { _ in },
            pairingProposal: .mock(proposal: .mock(metadata: .mock(url: "http://app.example.com"))),
        )

        #expect(model.imageUrl == nil)
    }

    @Test
    func appTextKeepsNameAndDomain() {
        let metadata = ApplicationMetadata.mock(
            name: "PancakeSwap - Trade",
            url: "https://pancakeswap.finance/swap",
        )
        let model = ConnectionProposalViewModel(
            confirmTransferDelegate: { _ in },
            pairingProposal: .mock(proposal: .mock(metadata: metadata)),
        )

        #expect(model.appText == "PancakeSwap (pancakeswap.finance)")
    }
}
