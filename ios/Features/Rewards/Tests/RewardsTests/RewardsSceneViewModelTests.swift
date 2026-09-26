// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Gemstone
import GemstonePrimitivesTestKit
import Localization
import Primitives
import PrimitivesTestKit
@testable import Rewards
import RewardsTestKit
import Testing

@MainActor
struct RewardsSceneViewModelTests {
    private let first = Wallet.mock(id: .mock(address: "0xaaa"), name: "First")
    private let second = Wallet.mock(id: .mock(address: "0xbbb"), name: "Second")

    @Test
    func noSelectedWalletMeansNoScene() {
        #expect(RewardsSceneViewModel.mock(wallets: []) == nil)
    }

    @Test
    func theSceneOffersTheWalletsCoreReturned() throws {
        let model = try #require(RewardsSceneViewModel.mock(wallets: [first, second]))

        #expect(model.selectedWallet.id == first.id)
        #expect(model.wallets.map(\.id) == [first.id, second.id])
        #expect(model.showsWalletSelector)
    }

    @Test
    func aSingleWalletHidesTheSelector() throws {
        let model = try #require(RewardsSceneViewModel.mock(wallets: [first]))

        #expect(model.showsWalletSelector == false)
    }

    @Test
    func loadingAsksCoreForTheSelectedWallet() async throws {
        let service = GemRewardsServiceMock()
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first, second]))

        await model.refresh()

        #expect(service.rewardsCalls == [first.id.id])
        #expect(model.rewardsState.referralCode == "test123")
        #expect(model.referralLink == "https://gemwallet.com/join?code=test123")
    }

    @Test
    func aFailedLoadShowsTheErrorInsteadOfTheCreateCodeScreen() async throws {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .failure(AnyError("offline"))
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first, second]))

        await model.refresh()

        guard case .error = model.viewState.state else {
            Issue.record("a failed load must not read as a wallet without a code")
            return
        }
        #expect(model.rewardsState.referralCode == nil)
        #expect(model.shareText == nil)
    }

    @Test
    func aLoadForAWalletThatIsNoLongerSelectedIsDropped() async throws {
        let service = GemRewardsServiceMock()
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first, second]))
        await model.refresh()

        model.selectWallet(id: second.id.id)
        await model.refresh()

        #expect(model.session.walletId == second.id.id)
    }

    @Test
    func oneWalletWithAnActivateCodeRedeemsItWithoutTheSheet() async throws {
        let service = GemRewardsServiceMock()
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first], activateCode: "friend"))

        await model.onTaskOnce()

        #expect(service.usedReferralCodes.map(\.code) == ["friend"])
        #expect(model.isPresentingSheet == nil)
    }

    @Test
    func severalWalletsWithAnActivateCodeAskWhichWalletFirst() async throws {
        let service = GemRewardsServiceMock()
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first, second], activateCode: "friend"))

        await model.onTaskOnce()

        #expect(service.usedReferralCodes.isEmpty)
        #expect(model.isPresentingSheet?.id == RewardsSheetType.activateCode(code: "friend").id)
    }

    @Test
    func selectingAWalletSwitchesTheSelection() throws {
        let model = try #require(RewardsSceneViewModel.mock(wallets: [first, second]))

        model.selectWallet(id: second.id.id)

        #expect(model.selectedWallet.id == second.id)
    }

    @Test
    func selectingAnUnknownWalletKeepsTheSelection() throws {
        let model = try #require(RewardsSceneViewModel.mock(wallets: [first, second]))

        model.selectWallet(id: "multicoin_0xccc")

        #expect(model.selectedWallet.id == first.id)
    }

    @Test
    func activatingAPendingReferralSendsTheStoredCode() async throws {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .success(.mock(code: nil, usedReferralCode: "pending", status: .pending, verifyAfter: .distantPast))
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first, second]))
        await model.refresh()

        await model.activatePendingReferral()

        #expect(service.usedReferralCodes.map(\.code) == ["pending"])
        #expect(model.toastMessage != nil)
        #expect(model.isPresentingAlert == nil)
    }

    @Test
    func aFailedActivationShowsTheError() async throws {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .success(.mock(code: nil, usedReferralCode: "pending", status: .pending, verifyAfter: .distantPast))
        service.useReferralCodeError = GemServiceError.Api(msg: "code already used")
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first, second]))
        await model.refresh()

        await model.activatePendingReferral()

        #expect(model.isPresentingAlert?.message == "code already used")
        #expect(model.toastMessage == nil)
    }

    @Test
    func redeemingSendsTheOptionId() async throws {
        let service = GemRewardsServiceMock()
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first, second]))

        await model.redeem(redemptionId: "option-7")

        #expect(service.redeemedIds == ["option-7"])
        #expect(model.toastMessage != nil)
    }

    @Test
    func aFailedRedemptionShowsTheError() async throws {
        let service = GemRewardsServiceMock()
        service.redeemError = GemServiceError.Api(msg: "out of stock")
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first, second]))

        await model.redeem(redemptionId: "option-7")

        #expect(model.isPresentingAlert?.message == "out of stock")
        #expect(model.toastMessage == nil)
    }

    @Test
    func aReadyPendingReferralCanBeActivated() async throws {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .success(.mock(code: nil, usedReferralCode: "pending", status: .pending, verifyAfter: .distantPast))
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first, second]))
        await model.refresh()

        #expect(model.activatePendingButtonType == .primary())
    }

    @Test
    func aWaitingPendingReferralCannotBeActivated() async throws {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .success(.mock(code: nil, usedReferralCode: "pending", status: .pending, verifyAfter: .distantFuture))
        let model = try #require(RewardsSceneViewModel.mock(service: service, wallets: [first, second]))
        await model.refresh()

        #expect(model.activatePendingButtonType == .primary(.disabled))
    }

    @Test
    func pendingNoticeTextsReadTheLocalizedCopy() {
        #expect(GemLocalizedText.rewardsPendingReady.text == Localized.Rewards.Pending.descriptionReady)
        #expect(GemLocalizedText.rewardsUnverified.text == Localized.Rewards.Unverified.description)
        #expect(GemLocalizedText.text(text: "verification required").text == "verification required")
    }
}
