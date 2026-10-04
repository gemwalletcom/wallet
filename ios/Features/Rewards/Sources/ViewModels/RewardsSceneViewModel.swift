// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemIncomingCode
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import enum Gemstone.GemRewardsIntroItem
import enum Gemstone.GemRewardsInviteAction
import struct Gemstone.GemRewardsPendingReferral
import struct Gemstone.GemRewardsRedemption
import protocol Gemstone.GemRewardsServiceProtocol
import struct Gemstone.GemRewardsSession
import struct Gemstone.GemRewardsState
import struct Gemstone.GemRewardsViewState
import struct Gemstone.GemRewardsWallet
import enum Gemstone.GemServiceError
import func Gemstone.rewardsSession
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Style

@Observable
@MainActor
public final class RewardsSceneViewModel: Sendable {
    private let service: any GemRewardsServiceProtocol

    private(set) var session: GemRewardsSession {
        didSet { viewState = session.viewState(now: Date()) }
    }

    private(set) var viewState: GemRewardsViewState
    var toastMessage: ToastMessage?
    var isPresentingSheet: RewardsSheetType?
    var isPresentingAlert: AlertMessage?

    public init(
        service: any GemRewardsServiceProtocol,
        wallets: [Wallet],
        currentWallet: Wallet?,
        activateCode: String? = nil,
    ) {
        self.service = service
        let session = rewardsSession(code: activateCode).onWallets(wallets: wallets.map { $0.toGem() }, current: currentWallet?.id)
        self.session = session
        viewState = session.viewState(now: Date())
    }

    // MARK: - UI Properties

    var title: String {
        Localized.Rewards.title
    }

    var errorTitle: String {
        Localized.Errors.errorOccurred
    }

    var createCodeButtonTitle: String {
        Localized.Common.getStarted
    }

    var createCodeTitle: String {
        Localized.Rewards.InviteFriends.title
    }

    var createCodeDescription: String {
        rewardsState.inviteDescription.text
    }

    var activateCodeFooterTitle: String {
        Localized.Rewards.ActivateReferralCode.title
    }

    var activateCodeFooterDescription: String {
        Localized.Rewards.ActivateReferralCode.description
    }

    var wallet: GemRewardsWallet? {
        viewState.wallet
    }

    var emptyContentModel: EmptyStateViewModel {
        EmptyStateViewModel(kind: .rewards)
    }

    var walletSelectorModel: SelectWalletViewModel? {
        wallet.map { SelectWalletViewModel(sections: $0.sections, selectedRow: $0.row) }
    }

    var shareText: String? {
        rewardsState.shareText?.text
    }

    var referralLink: String? {
        rewardsState.referralLink
    }

    var redemptionOptions: [GemRewardsRedemption] {
        rewardsState.redemptions
    }

    var rewardsState: GemRewardsState {
        viewState.rewards
    }

    var sections: [ListSection<GemListSectionRow>] {
        rewardsState.sections.listSections
    }

    var introItems: [GemRewardsIntroItem] {
        rewardsState.intro
    }

    var inviteAction: GemRewardsInviteAction? {
        rewardsState.inviteAction
    }

    var canUseReferralCode: Bool {
        rewardsState.canUseReferralCode
    }

    var pendingReferral: GemRewardsPendingReferral? {
        rewardsState.pendingReferral
    }

    var pendingReferralButtonTitle: String {
        Localized.Transfer.confirm
    }

    var activatePendingButtonType: ButtonType {
        pendingReferral?.isEnabled == true ? .primary() : .primary(.disabled)
    }

    var rewardsUrl: URL {
        AppUrl.rewards(.rewards)
    }

    var createCodeViewModel: CreateRewardsCodeViewModel? {
        wallet.map { wallet in
            CreateRewardsCodeViewModel(service: service, walletId: wallet.id) { [weak self] rewards in
                guard let self else { return }
                session = session.onRewards(walletId: wallet.id, rewards: rewards)
            }
        }
    }

    func redeemCodeViewModel(code: String) -> RedeemRewardsCodeViewModel? {
        wallet.map { wallet in
            RedeemRewardsCodeViewModel(service: service, walletId: wallet.id, code: code) { [weak self] rewards in
                guard let self else { return }
                session = session.onRewards(walletId: wallet.id, rewards: rewards)
                showActivatedToast()
            }
        }
    }

    // MARK: - Actions

    func onInviteFriends() {
        guard service.isAvailable() else {
            isPresentingSheet = .info(InfoSheetModel(sheet: GemInfoTopic.regionUnavailable.infoSheet))
            return
        }
        isPresentingSheet = .share
    }

    func selectWallet(id: String) {
        session = session.onSelectWallet(rowId: id)
        Task { await refresh() }
    }

    func refresh() async {
        if let walletId = session.wallet?.id {
            session = await session.onResult(result: service.refresh(walletId: walletId))
        }
        guard let code = viewState.incomingCode else { return }
        session = session.onCodeHandled()
        switch code {
        case let .activate(code): await useReferralCode(code)
        case let .confirm(code): isPresentingSheet = .activateCode(code: code)
        }
    }

    func activatePendingReferral() async {
        guard let code = pendingReferral?.code else { return }
        await useReferralCode(code)
    }

    private func useReferralCode(_ code: String) async {
        guard let walletId = session.wallet?.id else { return }
        do {
            let rewards = try await service.useReferralCode(walletId: walletId, code: code)
            session = session.onRewards(walletId: walletId, rewards: rewards)
            showActivatedToast()
        } catch let error as GemServiceError {
            showError(error.localizedDescription)
        } catch {
            debugLog("rewards error: \(error)")
        }
    }

    func onSelectRedemption(_ redemption: GemRewardsRedemption) {
        if redemption.canRedeem {
            showRedemptionAlert(for: redemption)
        } else {
            showError(Localized.Rewards.insufficientPoints)
        }
    }

    func showRedemptionAlert(for redemption: GemRewardsRedemption) {
        isPresentingAlert = AlertMessage(
            title: redemption.confirmation.text,
            message: "",
            actions: [
                AlertAction(title: Localized.Transfer.confirm, isDefaultAction: true) { [weak self] in
                    Task {
                        await self?.redeem(redemptionId: redemption.id)
                        await self?.refresh()
                    }
                },
                .cancel(title: Localized.Common.cancel),
            ],
        )
    }

    func redeem(redemptionId: String) async {
        guard let walletId = session.wallet?.id else { return }
        do {
            _ = try await service.redeem(walletId: walletId, redemptionId: redemptionId)
            toastMessage = ToastMessage.success(Localized.Common.done)
        } catch let error as GemServiceError {
            showError(error.localizedDescription)
        } catch {
            debugLog("rewards error: \(error)")
        }
    }

    private func showActivatedToast() {
        toastMessage = ToastMessage.success(Localized.Common.done)
    }

    func showError(_ message: String) {
        isPresentingAlert = AlertMessage(
            title: Localized.Errors.errorOccurred,
            message: message,
            actions: [.cancel(title: Localized.Common.done)],
        )
    }
}
