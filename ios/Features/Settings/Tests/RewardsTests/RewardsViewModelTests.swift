// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Gemstone
import GemstonePrimitivesTestKit
import Localization
import Primitives
import PrimitivesTestKit
import Testing
@testable import Settings
import SettingsTestKit

@MainActor
struct RewardsViewModelTests {
    private let first = Wallet.mock(id: .mock(address: "0xaaa"), name: "First")
    private let second = Wallet.mock(id: .mock(address: "0xbbb"), name: "Second")

    @Test
    func noSelectedWalletMeansNoScene() {
        #expect(RewardsViewModel.mock(wallets: []) == nil)
    }

    @Test
    func theSceneOffersTheWalletsCoreReturned() throws {
        let model = try #require(RewardsViewModel.mock(wallets: [first, second]))

        #expect(model.selectedWallet.id == first.id)
        #expect(model.wallets.map(\.id) == [first.id, second.id])
        #expect(model.showsWalletSelector)
    }

    @Test
    func aSingleWalletHidesTheSelector() throws {
        let model = try #require(RewardsViewModel.mock(wallets: [first]))

        #expect(model.showsWalletSelector == false)
    }

    @Test
    func loadingAsksCoreForTheSelectedWallet() async throws {
        let service = GemRewardsServiceMock()
        let model = try #require(RewardsViewModel.mock(service: service, wallets: [first, second]))

        await model.load()

        #expect(service.rewardsCalls == [first.id.id])
        #expect(model.referralCode == "test123")
        #expect(model.referralLink == "https://gemwallet.com/join?code=test123")
    }

    @Test
    func aFailedLoadFallsBackToTheEmptyState() async throws {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .failure(AnyError("offline"))
        let model = try #require(RewardsViewModel.mock(service: service, wallets: [first, second]))

        await model.load()

        #expect(model.state.isNoData)
        #expect(model.referralCode == nil)
        #expect(model.referralLink == nil)
        #expect(model.shareText == nil)
    }

    @Test
    func oneWalletWithAnActivateCodeRedeemsItWithoutTheSheet() async throws {
        let service = GemRewardsServiceMock()
        let model = try #require(RewardsViewModel.mock(service: service, wallets: [first], activateCode: "friend"))

        await model.onTaskOnce()

        #expect(service.usedReferralCodes.map { $0.code } == ["friend"])
        #expect(model.isPresentingSheet == nil)
    }

    @Test
    func severalWalletsWithAnActivateCodeAskWhichWalletFirst() async throws {
        let service = GemRewardsServiceMock()
        let model = try #require(RewardsViewModel.mock(service: service, wallets: [first, second], activateCode: "friend"))

        await model.onTaskOnce()

        #expect(service.usedReferralCodes.isEmpty)
        #expect(model.isPresentingSheet?.id == RewardsSheetType.activateCode(code: "friend").id)
    }

    @Test
    func selectingAWalletSwitchesTheSelection() throws {
        let model = try #require(RewardsViewModel.mock(wallets: [first, second]))

        model.selectWallet(id: second.id.id)

        #expect(model.selectedWallet.id == second.id)
    }

    @Test
    func selectingAnUnknownWalletKeepsTheSelection() throws {
        let model = try #require(RewardsViewModel.mock(wallets: [first, second]))

        model.selectWallet(id: "multicoin_0xccc")

        #expect(model.selectedWallet.id == first.id)
    }

    @Test
    func activatingAPendingReferralSendsTheStoredCode() async throws {
        let service = GemRewardsServiceMock()
        service.stateForRewards = { _ in .mock(canActivatePendingReferral: true, usedReferralCode: "pending") }
        let model = try #require(RewardsViewModel.mock(service: service, wallets: [first, second]))
        await model.load()

        await model.activatePendingReferral()

        #expect(service.usedReferralCodes.map { $0.code } == ["pending"])
        #expect(model.toastMessage != nil)
        #expect(model.isPresentingAlert == nil)
    }

    @Test
    func aFailedActivationShowsTheError() async throws {
        let service = GemRewardsServiceMock()
        service.stateForRewards = { _ in .mock(canActivatePendingReferral: true, usedReferralCode: "pending") }
        service.useReferralCodeError = GemServiceError.Api(msg: "code already used")
        let model = try #require(RewardsViewModel.mock(service: service, wallets: [first, second]))
        await model.load()

        await model.activatePendingReferral()

        #expect(model.isPresentingAlert?.message == "code already used")
        #expect(model.toastMessage == nil)
    }

    @Test
    func redeemingSendsTheOptionId() async throws {
        let service = GemRewardsServiceMock()
        let model = try #require(RewardsViewModel.mock(service: service, wallets: [first, second]))

        await model.redeem(option: .mock(id: "option-7"))

        #expect(service.redeemedIds == ["option-7"])
        #expect(model.toastMessage != nil)
    }

    @Test
    func aFailedRedemptionShowsTheError() async throws {
        let service = GemRewardsServiceMock()
        service.redeemError = GemServiceError.Api(msg: "out of stock")
        let model = try #require(RewardsViewModel.mock(service: service, wallets: [first, second]))

        await model.redeem(option: .mock(id: "option-7"))

        #expect(model.isPresentingAlert?.message == "out of stock")
        #expect(model.toastMessage == nil)
    }

    @Test
    func aReadyPendingReferralReadsAsReady() async throws {
        let service = GemRewardsServiceMock()
        service.stateForRewards = { _ in .mock(hasPendingReferral: true, canActivatePendingReferral: true, usedReferralCode: "pending") }
        let model = try #require(RewardsViewModel.mock(service: service, wallets: [first, second]))
        await model.load()

        #expect(model.pendingReferralDescription == Localized.Rewards.Pending.descriptionReady)
        #expect(model.activatePendingButtonType == .primary())
    }

    @Test
    func aPendingReferralWithNoDateHasNoDescription() async throws {
        let service = GemRewardsServiceMock()
        service.stateForRewards = { _ in .mock(hasPendingReferral: true, usedReferralCode: "pending") }
        let model = try #require(RewardsViewModel.mock(service: service, wallets: [first, second]))
        await model.load()

        #expect(model.pendingReferralDescription == nil)
        #expect(model.activatePendingButtonType == .primary(.disabled))
    }
}
