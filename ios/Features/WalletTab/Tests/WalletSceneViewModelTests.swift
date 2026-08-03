// Copyright (c). Gem Wallet. All rights reserved.

import BannerServiceTestKit
import Localization
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing
@testable import WalletTab
import WalletTabTestKit

@MainActor
struct WalletSceneViewModelTests {
    @Test
    func priorityBannerReturnsHighestPriority() {
        let model = WalletSceneViewModel.mock()
        model.bannersQuery.value = [
            .mock(event: .stake, state: .active),
            .mock(event: .enableNotifications, state: .cancelled),
            .mock(event: .accountActivation, state: .alwaysActive),
        ]

        #expect(model.walletBannersModel.allBanners.first?.state == .alwaysActive)
    }

    @Test
    func onChangeWalletUpdatesImageUrl() {
        let wallet = Wallet.mock(id: .multicoin(address: "0x1"), imageUrl: nil)
        let model = WalletSceneViewModel.mock(wallet: wallet)

        #expect(model.wallet.imageUrl == nil)

        let updatedWallet = Wallet.mock(id: .multicoin(address: "0x1"), imageUrl: "avatar.png")
        model.onChangeWallet(wallet, updatedWallet)

        #expect(model.wallet.imageUrl == "avatar.png")
    }

    @Test
    func onChangeWalletSwitchesToDifferentWallet() {
        let wallet = Wallet.mock(id: .multicoin(address: "0x1"), name: "Wallet 1")
        let model = WalletSceneViewModel.mock(wallet: wallet)

        #expect(model.wallet.id == .multicoin(address: "0x1"))

        let newWallet = Wallet.mock(id: .multicoin(address: "0x2"), name: "Wallet 2")
        model.onChangeWallet(wallet, newWallet)

        #expect(model.wallet.id == .multicoin(address: "0x2"))
    }

    @Test
    func onHandleScan() {
        let model = WalletSceneViewModel.mock()

        model.onHandleScan("https://pay.walletconnect.com/?pid=pay_123")
        #expect(model.isPresentingToastMessage == nil)

        model.onHandleScan("wc:abc@2?relay-protocol=irn&symKey=123")
        #expect(model.isPresentingToastMessage?.title == Localized.Errors.notSupported)

        model.onHandleScan("WIFI:S:MyNet;T:WPA;P:secret;;")
        #expect(model.isPresentingToastMessage?.title == Localized.Errors.notSupported)
    }
}
