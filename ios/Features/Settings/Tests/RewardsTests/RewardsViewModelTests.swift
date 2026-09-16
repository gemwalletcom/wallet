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

@MainActor
struct RewardsViewModelTests {
    private let first = Wallet.mock(id: .mock(address: "0xaaa"), name: "First")
    private let second = Wallet.mock(id: .mock(address: "0xbbb"), name: "Second")

    private func service(_ configure: (GemRewardsServiceMock) -> Void = { _ in }) -> GemRewardsServiceMock {
        let service = GemRewardsServiceMock()
        service.selectedWalletValue = first.toGem()
        service.walletsValue = [first.toGem(), second.toGem()]
        configure(service)
        return service
    }

    private func viewModel(_ service: GemRewardsServiceMock, activateCode: String? = nil) -> RewardsViewModel? {
        RewardsViewModel(service: service, wallets: [first, second], currentWallet: first, activateCode: activateCode)
    }

    @Test
    func noSelectedWalletMeansNoScene() {
        let service = GemRewardsServiceMock()
        service.selectedWalletValue = nil

        #expect(RewardsViewModel(service: service, wallets: [], currentWallet: nil) == nil)
    }

    @Test
    func theSceneOffersTheWalletsCoreReturned() throws {
        let model = try #require(viewModel(service()))

        #expect(model.selectedWallet.id == first.id)
        #expect(model.wallets.map(\.id) == [first.id, second.id])
        #expect(model.showsWalletSelector)
    }

    @Test
    func aSingleWalletHidesTheSelector() throws {
        let service = service { $0.walletsValue = [first.toGem()] }
        let model = try #require(RewardsViewModel(service: service, wallets: [first], currentWallet: first))

        #expect(model.showsWalletSelector == false)
    }

    @Test
    func loadingAsksCoreForTheSelectedWallet() async throws {
        let service = service()
        let model = try #require(viewModel(service))

        await model.load()

        #expect(service.rewardsCalls == [first.id.id])
        #expect(model.referralCode == "test123")
        #expect(model.referralLink == "https://gemwallet.com/join?code=test123")
    }

    @Test
    func aFailedLoadFallsBackToTheEmptyState() async throws {
        let service = service {
            $0.rewardsResult = .failure(AnyError("offline"))
            $0.stateForRewards = { rewards in .mock(referralCode: rewards?.code, referralLink: rewards?.code.map { "https://gemwallet.com/join?code=\($0)" }) }
        }
        let model = try #require(viewModel(service))

        await model.load()

        #expect(model.state.isNoData)
        #expect(model.referralCode == nil)
        #expect(model.referralLink == nil)
        #expect(model.shareText == nil)
    }

    @Test
    func oneWalletWithAnActivateCodeRedeemsItWithoutTheSheet() async throws {
        let service = service { $0.walletsValue = [first.toGem()] }
        let model = try #require(RewardsViewModel(service: service, wallets: [first], currentWallet: first, activateCode: "friend"))

        await model.onTaskOnce()

        #expect(service.usedReferralCodes.map { $0.code } == ["friend"])
        #expect(model.isPresentingSheet == nil)
    }

    @Test
    func severalWalletsWithAnActivateCodeAskWhichWalletFirst() async throws {
        let service = service()
        let model = try #require(viewModel(service, activateCode: "friend"))

        await model.onTaskOnce()

        #expect(service.usedReferralCodes.isEmpty)
        #expect(model.isPresentingSheet?.id == RewardsSheetType.activateCode(code: "friend").id)
    }

    @Test
    func selectingAWalletSwitchesTheSelection() throws {
        let model = try #require(viewModel(service()))

        model.selectWallet(id: second.id.id)

        #expect(model.selectedWallet.id == second.id)
    }

    @Test
    func selectingAnUnknownWalletKeepsTheSelection() throws {
        let model = try #require(viewModel(service()))

        model.selectWallet(id: "multicoin_0xccc")

        #expect(model.selectedWallet.id == first.id)
    }

    @Test
    func activatingAPendingReferralSendsTheStoredCode() async throws {
        let service = service { $0.stateForRewards = { _ in .mock(canActivatePendingReferral: true, usedReferralCode: "pending") } }
        let model = try #require(viewModel(service))
        await model.load()

        await model.activatePendingReferral()

        #expect(service.usedReferralCodes.map { $0.code } == ["pending"])
        #expect(model.toastMessage != nil)
        #expect(model.isPresentingAlert == nil)
    }

    @Test
    func aFailedActivationShowsTheError() async throws {
        let service = service {
            $0.stateForRewards = { _ in .mock(canActivatePendingReferral: true, usedReferralCode: "pending") }
            $0.useReferralCodeError = AnyError("code already used")
        }
        let model = try #require(viewModel(service))
        await model.load()

        await model.activatePendingReferral()

        #expect(model.isPresentingAlert?.message == "code already used")
        #expect(model.toastMessage == nil)
    }

    @Test
    func redeemingSendsTheOptionId() async throws {
        let service = service()
        let model = try #require(viewModel(service))

        await model.redeem(option: .mock(id: "option-7"))

        #expect(service.redeemedIds == ["option-7"])
        #expect(model.toastMessage != nil)
    }

    @Test
    func aFailedRedemptionShowsTheError() async throws {
        let service = service { $0.redeemError = AnyError("out of stock") }
        let model = try #require(viewModel(service))

        await model.redeem(option: .mock(id: "option-7"))

        #expect(model.isPresentingAlert?.message == "out of stock")
        #expect(model.toastMessage == nil)
    }

    @Test
    func aReadyPendingReferralReadsAsReady() async throws {
        let service = service {
            $0.stateForRewards = { _ in .mock(canActivatePendingReferral: true, usedReferralCode: "pending", verifyAfter: Date(timeIntervalSince1970: 0)) }
        }
        let model = try #require(viewModel(service))
        await model.load()

        #expect(model.pendingReferralDescription == Localized.Rewards.Pending.descriptionReady)
        #expect(model.activatePendingButtonType == .primary())
    }

    @Test
    func aPendingReferralWithNoDateHasNoDescription() async throws {
        let service = service { $0.stateForRewards = { _ in .mock(usedReferralCode: "pending", verifyAfter: nil) } }
        let model = try #require(viewModel(service))
        await model.load()

        #expect(model.pendingReferralDescription == nil)
        #expect(model.activatePendingButtonType == .primary(.disabled))
    }
}
