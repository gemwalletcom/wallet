// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Gemstone
import GemstonePrimitivesTestKit
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
    func noMulticoinWalletShowsTheEmptyState() async {
        let model = RewardsSceneViewModel.mock(service: GemRewardsServiceMock(), wallets: [], activateCode: "friend")

        await model.refresh()

        #expect(model.viewState.state == .noData)
        #expect(model.wallet == nil)
        #expect(model.isPresentingSheet == nil)
    }

    @Test
    func theSceneOffersTheWalletsCoreReturned() async {
        let model = RewardsSceneViewModel.mock(service: GemRewardsServiceMock(), wallets: [first, second])

        await model.refresh()

        #expect(model.wallet?.id == first.id)
        #expect(model.walletSelectorModel?.selectedItems.map(\.id) == [first.id.id])
        #expect(model.wallet?.canChoose == true)
    }

    @Test
    func theWalletSelectorIsReadyBeforeTheFirstLoad() {
        let model = RewardsSceneViewModel.mock(service: GemRewardsServiceMock(), wallets: [first, second])

        #expect(model.wallet?.id == first.id)
        #expect(model.wallet?.canChoose == true)
        #expect(model.viewState.state == .loading)
    }

    @Test
    func aSingleWalletHidesTheSelector() async {
        let model = RewardsSceneViewModel.mock(service: GemRewardsServiceMock(), wallets: [first])

        await model.refresh()

        #expect(model.wallet?.canChoose == false)
    }

    @Test
    func loadingAsksCoreForTheChosenWallet() async {
        let service = GemRewardsServiceMock()
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first, second])

        await model.refresh()
        #expect(service.rewardsCalls == [first.id])

        model.selectWallet(id: second.id.id)
        await model.refresh()

        #expect(service.rewardsCalls.last == second.id)
        #expect(model.wallet?.id == second.id)
        #expect(model.rewardsState.referralCode == "test123")
        #expect(model.referralLink == "https://gemwallet.com/join?code=test123")
    }

    @Test
    func aFailedLoadShowsTheErrorInsteadOfTheCreateCodeScreen() async {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .failure(AnyError("offline"))
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first, second])

        await model.refresh()

        guard case .error = model.viewState.state else {
            Issue.record("a failed load must not read as a wallet without a code")
            return
        }
        #expect(model.rewardsState.referralCode == nil)
        #expect(model.shareText == nil)
    }

    @Test
    func oneWalletWithAnActivateCodeRedeemsItOnceWithoutTheSheet() async {
        let service = GemRewardsServiceMock()
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first], activateCode: "friend")

        await model.refresh()
        await model.refresh()

        #expect(service.usedReferralCodes.map(\.code) == ["friend"])
        #expect(service.usedReferralCodes.map(\.walletId) == [first.id])
        #expect(model.isPresentingSheet == nil)
    }

    @Test
    func severalWalletsWithAnActivateCodeAskWhichWalletFirst() async {
        let service = GemRewardsServiceMock()
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first, second], activateCode: "friend")

        await model.refresh()

        #expect(service.usedReferralCodes.isEmpty)
        #expect(model.isPresentingSheet?.id == RewardsSheetType.activateCode(code: "friend").id)
        #expect(model.viewState.incomingCode == nil)
    }

    @Test
    func aCodeRedeemedFromTheSheetShowsTheStateItProduced() async throws {
        let service = GemRewardsServiceMock()
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first, second])
        await model.refresh()
        service.rewardsResult = .success(.mock(code: nil, usedReferralCode: "friend", status: .pending, verifyAfter: .distantFuture))
        let sheet = try #require(model.redeemCodeViewModel(code: "friend"))

        await sheet.action()

        #expect(model.pendingReferral?.code == "friend")
        #expect(service.rewardsCalls.count == 1, "the sheet's answer is the new state, no reload")
        #expect(model.toastMessage != nil)
    }

    @Test
    func activatingAPendingReferralSendsTheStoredCode() async {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .success(.mock(code: nil, usedReferralCode: "pending", status: .pending, verifyAfter: .distantPast))
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first, second])
        await model.refresh()

        await model.activatePendingReferral()

        #expect(service.usedReferralCodes.map(\.code) == ["pending"])
        #expect(model.toastMessage != nil)
        #expect(model.isPresentingAlert == nil)
    }

    @Test
    func aFailedActivationShowsTheError() async {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .success(.mock(code: nil, usedReferralCode: "pending", status: .pending, verifyAfter: .distantPast))
        service.useReferralCodeError = GemServiceError.Api(msg: "code already used")
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first, second])
        await model.refresh()

        await model.activatePendingReferral()

        #expect(model.isPresentingAlert?.message == "code already used")
        #expect(model.toastMessage == nil)
    }

    @Test
    func redeemingSendsTheOptionId() async {
        let service = GemRewardsServiceMock()
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first, second])
        await model.refresh()

        await model.redeem(redemptionId: "option-7")

        #expect(service.redeemedIds == ["option-7"])
        #expect(model.toastMessage != nil)
    }

    @Test
    func aFailedRedemptionShowsTheError() async {
        let service = GemRewardsServiceMock()
        service.redeemError = GemServiceError.Api(msg: "out of stock")
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first, second])
        await model.refresh()

        await model.redeem(redemptionId: "option-7")

        #expect(model.isPresentingAlert?.message == "out of stock")
        #expect(model.toastMessage == nil)
    }

    @Test
    func aReadyPendingReferralCanBeActivated() async {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .success(.mock(code: nil, usedReferralCode: "pending", status: .pending, verifyAfter: .distantPast))
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first, second])
        await model.refresh()

        #expect(model.activatePendingButtonType == .primary())
    }

    @Test
    func aWaitingPendingReferralCannotBeActivated() async {
        let service = GemRewardsServiceMock()
        service.rewardsResult = .success(.mock(code: nil, usedReferralCode: "pending", status: .pending, verifyAfter: .distantFuture))
        let model = RewardsSceneViewModel.mock(service: service, wallets: [first, second])
        await model.refresh()

        #expect(model.activatePendingButtonType == .primary(.disabled))
    }
}
