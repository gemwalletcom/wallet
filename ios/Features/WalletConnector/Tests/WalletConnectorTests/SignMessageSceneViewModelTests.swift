// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSimulationValue
import struct Gemstone.GemWalletConnectMessageRequest
import struct Gemstone.SimulationHeader
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import WalletConnector
import WalletConnectorService
import WalletConnectorServiceTestKit
import WalletConnectorTestKit

struct SignMessageSceneViewModelTests {
    private let permitBatch = Data("""
    {
      "types": {
        "EIP712Domain": [
          { "name": "name", "type": "string" },
          { "name": "chainId", "type": "uint256" },
          { "name": "verifyingContract", "type": "address" }
        ],
        "PermitBatch": [
          { "name": "details", "type": "PermitDetails[]" },
          { "name": "spender", "type": "address" },
          { "name": "sigDeadline", "type": "uint256" }
        ],
        "PermitDetails": [
          { "name": "token", "type": "address" },
          { "name": "amount", "type": "uint160" },
          { "name": "expiration", "type": "uint48" },
          { "name": "nonce", "type": "uint48" }
        ]
      },
      "primaryType": "PermitBatch",
      "domain": {
        "name": "Permit2",
        "chainId": "1",
        "verifyingContract": "0x000000000022D473030F116dDEE9F6B43aC78BA3"
      },
      "message": {
        "details": [
          {
            "token": "0x1111111111111111111111111111111111111111",
            "amount": "1000000000000000000",
            "expiration": "1712600000",
            "nonce": "0"
          }
        ],
        "spender": "0x3333333333333333333333333333333333333333",
        "sigDeadline": "1712600500"
      }
    }
    """.utf8)

    @Test
    @MainActor
    func walletTextDisplaysPayloadWallet() {
        let wallet = Wallet.mock(name: "My Secure Wallet")
        let payload = GemWalletConnectMessageRequest.mock(wallet: wallet.toGem())

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        guard case let .wallet(_, row, _) = viewModel.rows.first else {
            Issue.record("expected the wallet row first without a header")
            return
        }
        #expect(row.name == "My Secure Wallet")
    }

    @Test
    @MainActor
    func appTextUsesShortNameWithoutDomain() {
        let payload = GemWalletConnectMessageRequest.mock(
            session: WalletConnectionSession.mock(metadata: .mock(
                name: "PancakeSwap - Trade",
                url: "https://pancakeswap.finance/swap",
            )).toGem(),
        )

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        #expect(viewModel.appName == "PancakeSwap")
    }

    @Test
    @MainActor
    func titleUsesReviewRequest() {
        let viewModel = SignMessageSceneViewModel.mock()

        #expect(viewModel.title == "Review Request")
    }

    @Test
    @MainActor
    func payloadStoresValidatedChainNotMessageChain() {
        let payload = GemWalletConnectMessageRequest.mock(chain: Chain.ethereum.rawValue, message: .mock(chain: "bitcoin"))

        #expect(payload.chain == Chain.ethereum.rawValue)
        #expect(payload.message.chain == "bitcoin")
    }

    @Test
    @MainActor
    func networkTextUsesPayloadChain() {
        let payload = GemWalletConnectMessageRequest.mock(chain: Chain.ethereum.rawValue, message: .mock(chain: "bitcoin"))

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        guard case let .network(_, chain, name) = viewModel.rows.last else {
            Issue.record("expected the network row last")
            return
        }
        #expect(chain == Chain.ethereum.rawValue)
        #expect(name == "Ethereum")
    }

    @Test
    @MainActor
    func buttonEnabledWithNoWarnings() {
        let viewModel = SignMessageSceneViewModel.mock()

        #expect(!viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func buttonEnabledWithNonCriticalWarnings() {
        let payload = GemWalletConnectMessageRequest.mock(
            simulation: .mock(warnings: [.mock(severity: .warning)]),
        )

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        #expect(!viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func simulationWarningsHideBoundedApprovals() {
        let payload = GemWalletConnectMessageRequest.mock(
            simulation: .mock(warnings: [
                .mock(severity: .warning, warning: .tokenApproval(.mock(value: 1000))),
                .mock(severity: .warning),
            ]),
        )

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        #expect(viewModel.simulationWarnings == [.notice(title: .unlimitedApproval, message: .unlimitedApprovalWarning, kind: .warning)])
    }

    @Test
    @MainActor
    func buttonDisabledWithCriticalWarnings() {
        let payload = GemWalletConnectMessageRequest.mock(
            simulation: .mock(warnings: [.mock(severity: .critical, warning: .suspiciousSpender)]),
        )

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        #expect(viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func simulationWarningsHideBoundedApprovalsAndKeepExternallyOwnedSpenderWarnings() {
        let payload = GemWalletConnectMessageRequest.mock(
            simulation: .mock(warnings: [
                .mock(severity: .warning, warning: .permitApproval(.mock(value: 1000))),
                .mock(severity: .warning, warning: .externallyOwnedSpender),
            ]),
        )

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        #expect(viewModel.simulationWarnings == [.notice(title: .warning, message: .externallyOwnedSpenderWarning, kind: .warning)])
        #expect(!viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func permitBatchExternallyOwnedSpenderKeepsWarningAndPayload() {
        let payload = GemWalletConnectMessageRequest.mock(
            simulation: .mock(warnings: [.mock(severity: .warning, warning: .externallyOwnedSpender)], payload: [
                .mock(kind: .method, value: "Permit Batch", fieldType: .text, display: .primary),
                .mock(kind: .contract, value: "0x000000000022D473030F116dDEE9F6B43aC78BA3", fieldType: .address, display: .primary),
                .mock(kind: .spender, value: "0x3333333333333333333333333333333333333333", fieldType: .address, display: .primary),
                .mock(kind: .value, value: "Unlimited", fieldType: .text, display: .primary),
            ]),
            message: .mock(chain: "ethereum", signType: .eip712, data: permitBatch),
        )

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        #expect(viewModel.simulationWarnings == [.notice(title: .warning, message: .externallyOwnedSpenderWarning, kind: .warning)])
        #expect(!viewModel.isButtonDisabled)
        #expect(viewModel.hasPayloadFields)
        #expect(viewModel.primaryPayloadFields.contains { row in
            guard case let .address(_, copy, _) = row.value else { return false }
            return row.title == .spender && copy.value == "0x3333333333333333333333333333333333333333"
        })
    }

    @Test
    @MainActor
    func simulationWarningsPassThroughValidationWarnings() {
        let payload = GemWalletConnectMessageRequest.mock(
            simulation: .mock(warnings: [
                .mock(severity: .warning, warning: .permitApproval(.mock(value: 1000))),
                .mock(severity: .critical, warning: .validationError, message: "Unable to verify spender is a contract"),
            ]),
        )

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        #expect(viewModel.simulationWarnings == [.notice(title: .error, message: .text(text: "Unable to verify spender is a contract"), kind: .error)])
    }

    @Test
    @MainActor
    func siweChainMismatchStillUsesStructuredPayload() {
        let message = [
            "thepoc.xyz wants you to sign in with your Ethereum account:",
            "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4",
            "",
            "Sign in with different chain ID",
            "",
            "URI: https://thepoc.xyz",
            "Version: 1",
            "Chain ID: 137",
            "Nonce: gv7zples2q60kq7bnamtuwo",
            "Issued At: 2026-03-11T04:20:21.742Z",
        ]
        .joined(separator: "\n")

        let payload = GemWalletConnectMessageRequest.mock(
            simulation: .mock(warnings: [
                .mock(severity: .critical, warning: .validationError, message: "Chain ID mismatch"),
            ]),
            message: .mock(data: Data(message.utf8)),
        )

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        #expect(viewModel.title == "Sign In with Ethereum")
        #expect(viewModel.isButtonDisabled)
        #expect(viewModel.hasPayloadFields)
        #expect(viewModel.primaryPayloadFields.count == 2)
    }

    @Test
    @MainActor
    func permitHeaderReplacesValueField() {
        let asset = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)
        let payload = GemWalletConnectMessageRequest.mock(
            simulation: .mock(payload: [
                .mock(kind: .method, value: "Permit Batch", fieldType: .text, display: .primary),
                .mock(kind: .contract, value: "0x000000000022D473030F116dDEE9F6B43aC78BA3", fieldType: .address, display: .primary),
                .mock(kind: .spender, value: "0x3333333333333333333333333333333333333333", fieldType: .address, display: .primary),
                .mock(kind: .value, value: "Unlimited", fieldType: .text, display: .primary),
            ], header: SimulationHeader(assetId: asset.id.identifier, value: nil, isUnlimited: true)),
            message: .mock(chain: "ethereum", signType: .eip712, data: permitBatch),
            assets: [asset.toGem()],
        )

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        #expect(viewModel.headerData?.asset == asset.toGem())
        #expect(viewModel.headerData?.value == .unlimited)
        #expect(!(viewModel.primaryPayloadFields + viewModel.secondaryPayloadFields).contains { $0.title == .value })
    }

    @Test
    @MainActor
    func permitWithoutHeaderKeepsValueField() {
        let payload = GemWalletConnectMessageRequest.mock(
            simulation: .mock(payload: [
                .mock(kind: .method, value: "Permit Batch", fieldType: .text, display: .primary),
                .mock(kind: .contract, value: "0x000000000022D473030F116dDEE9F6B43aC78BA3", fieldType: .address, display: .primary),
                .mock(kind: .spender, value: "0x3333333333333333333333333333333333333333", fieldType: .address, display: .primary),
                .mock(kind: .value, value: "Unlimited", fieldType: .text, display: .primary),
            ]),
            message: .mock(chain: "ethereum", signType: .eip712, data: permitBatch),
        )

        let viewModel = SignMessageSceneViewModel.mock(request: payload)

        #expect(viewModel.headerData == nil)
        #expect((viewModel.primaryPayloadFields + viewModel.secondaryPayloadFields).contains { $0.title == .value })
    }
}
