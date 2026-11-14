// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.VerificationStatus
import struct Gemstone.GemWalletConnectAuthMessage
import func Gemstone.walletRow
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import WalletConnector
import WalletConnectorService
import WalletConnectorTestKit

struct AuthenticationSceneViewModelTests {
    @Test
    @MainActor
    func theReviewShowsTheSignInMessageOfTheChosenWallet() {
        let main = Wallet.mock(id: .mock(address: "main"), name: "Main")
        let other = Wallet.mock(id: .mock(address: "other"), name: "Other")
        let model = AuthenticationSceneViewModel.mock(request: request(defaultWallet: main, wallets: [main, other]) { wallet in
            GemWalletConnectAuthMessage(account: Account.mock(chain: .ethereum, address: "0x1111111111111111111111111111111111111111").toGem(), message: Self.signIn(statement: "Sign in as \(wallet.name)."))
        })

        #expect(model.hasPayloadFields, "the Sign-In fields show instead of raw text")
        #expect(model.buttonState == .normal)
        #expect(walletName(model) == "Main")
        #expect(model.messageText?.contains("Sign in as Main.") == true)

        model.walletSelectorModel.selectedItems = [walletRow(wallet: other.toGem())]

        #expect(walletName(model) == "Other")
        #expect(model.messageText?.contains("Sign in as Other.") == true, "a wallet chosen in the review signs in with its own account")
    }

    @Test
    @MainActor
    func aSignInCoreCannotPrepareCannotBeConfirmed() {
        let model = AuthenticationSceneViewModel.mock(request: request(defaultWallet: .mock()) { _ in throw AnyError("Requested chains are not supported") })

        #expect(model.buttonState == .disabled)
        #expect(model.isPresentingAlertMessage != nil)
    }

    @Test(arguments: [
        (WalletConnectionVerificationStatus.verified, Gemstone.VerificationStatus.verified),
        (.unknown, .unverified),
        (.invalid, .suspicious),
        (.malicious, .suspicious),
    ])
    @MainActor
    func theReviewShowsTheDappVerificationLevel(status: WalletConnectionVerificationStatus, expected: Gemstone.VerificationStatus) {
        let model = AuthenticationSceneViewModel.mock(request: request(defaultWallet: .mock(), verificationStatus: status) { _ in
            GemWalletConnectAuthMessage(account: Account.mock(chain: .ethereum, address: "0x1111111111111111111111111111111111111111").toGem(), message: Self.signIn(statement: "Sign in."))
        })

        #expect(model.sections.first?.items.last == .row(row: .verification(title: .status, status: expected, info: nil)))
    }

    private func request(defaultWallet: Wallet, wallets: [Wallet] = [], verificationStatus: WalletConnectionVerificationStatus = .verified,
                         message: @escaping @Sendable (Wallet) throws -> GemWalletConnectAuthMessage) -> WCAuthenticationRequest
    {
        WCAuthenticationRequest(
            id: "1",
            proposal: .mock(defaultWallet: defaultWallet, wallets: wallets.isEmpty ? [defaultWallet] : wallets),
            verificationStatus: verificationStatus,
            message: message,
            approve: { _, _ in },
        )
    }

    @MainActor
    private func walletName(_ model: AuthenticationSceneViewModel) -> String? {
        guard case let .row(.wallet(_, row, _, _)) = model.sections.first?.items.first else { return nil }
        return row.name
    }

    private static func signIn(statement: String) -> String {
        """
        example.com wants you to sign in with your Ethereum account:
        0x1111111111111111111111111111111111111111

        \(statement)

        URI: https://example.com/login
        Version: 1
        Chain ID: 1
        Nonce: 32891756
        Issued At: 2024-01-29T08:54:38Z
        """
    }
}
