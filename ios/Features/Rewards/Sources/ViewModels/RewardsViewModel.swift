// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemIncomingCode
import enum Gemstone.GemListRow
import enum Gemstone.GemRewardsAction
import struct Gemstone.GemRewardsRedemption
import protocol Gemstone.GemRewardsServiceProtocol
import struct Gemstone.GemRewardsSession
import struct Gemstone.GemRewardsState
import struct Gemstone.GemRewardsViewState
import enum Gemstone.GemServiceError
import struct Gemstone.GemWalletRow
import func Gemstone.incomingReferralCode
import func Gemstone.rewardsSession
import func Gemstone.walletRow
import func Gemstone.walletSections
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style

@Observable
@MainActor
public final class RewardsViewModel: Sendable {
    private let service: any GemRewardsServiceProtocol
    private let activateCode: String?

    private(set) var selectedWallet: Wallet {
        didSet { selectedWalletRow = walletRow(wallet: selectedWallet.toGem()) }
    }

    private(set) var selectedWalletRow: GemWalletRow
    private(set) var wallets: [Wallet]

    private(set) var session: GemRewardsSession {
        didSet { viewState = session.viewState(now: Date()) }
    }

    private(set) var viewState: GemRewardsViewState
    var toastMessage: ToastMessage?
    var isPresentingSheet: RewardsSheetType?
    var isPresentingAlert: AlertMessage?

    public init?(
        service: any GemRewardsServiceProtocol,
        wallets: [Wallet],
        currentWallet: Wallet?,
        activateCode: String? = nil,
    ) {
        let core = wallets.map { $0.toGem() }
        guard let wallet = service.selectedWallet(current: currentWallet?.toGem(), wallets: core).map({ $0.toPrimitives() }) else { return nil }
        self.service = service
        let session = rewardsSession().onSelectWallet(walletId: wallet.id.id)
        self.session = session
        viewState = session.viewState(now: Date())
        selectedWallet = wallet
        selectedWalletRow = walletRow(wallet: wallet.toGem())
        self.wallets = service.wallets(wallets: core).map { $0.toPrimitives() }
        self.activateCode = activateCode
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
        Localized.Rewards.InviteFriends.description(rewardsState.inviteRewardPoints.text().boldMarkdown())
    }

    var activateCodeFooterTitle: String {
        Localized.Rewards.ActivateReferralCode.title
    }

    var activateCodeFooterDescription: String {
        Localized.Rewards.ActivateReferralCode.description
    }

    var showsWalletSelector: Bool {
        wallets.count > 1
    }

    var walletSelectorModel: SelectWalletViewModel {
        SelectWalletViewModel(
            sections: walletSections(wallets: wallets.map { $0.toGem() }),
            selectedRow: selectedWalletRow,
        )
    }

    var shareText: String? {
        referralLink.map { Localized.Rewards.shareText($0) }
    }

    var referralLink: String? {
        rewardsState.referralLink
    }

    var redemptionOptions: [RewardRedemptionOptionViewModel] {
        rewardsState.redemptions.map { RewardRedemptionOptionViewModel(redemption: $0) }
    }

    var rewardsState: GemRewardsState {
        viewState.rewards
    }

    var sections: [ListSection<GemListSectionRow>] {
        rewardsState.sections.listSections
    }

    func action(_ action: GemRewardsAction) -> Bool {
        rewardsState.actions.contains(action)
    }

    var pendingReferral: (code: String, isEnabled: Bool)? {
        rewardsState.actions.compactMap { action -> (code: String, isEnabled: Bool)? in
            guard case let .activatePendingReferral(code, isEnabled) = action else { return nil }
            return (code, isEnabled)
        }.first
    }

    var pendingReferralButtonTitle: String {
        Localized.Transfer.confirm
    }

    var activatePendingButtonType: ButtonType {
        pendingReferral?.isEnabled == true ? .primary() : .primary(.disabled)
    }

    var walletBarViewModel: WalletBarViewViewModel {
        let row = selectedWalletRow
        return WalletBarViewViewModel(name: row.name, image: row.avatarImage)
    }

    var rewardsUrl: URL {
        AppUrl.rewards(.rewards)
    }

    var createCodeViewModel: CreateRewardsCodeViewModel {
        CreateRewardsCodeViewModel(
            service: service,
            wallet: selectedWallet,
        ) { [weak self] rewards in
            guard let self else { return }
            session = session.onRewards(rewards: rewards)
        }
    }

    func redeemCodeViewModel(code: String) -> RedeemRewardsCodeViewModel {
        RedeemRewardsCodeViewModel(
            service: service,
            wallet: selectedWallet,
            code: code,
        ) { [weak self] _ in
            guard let self else { return }
            showActivatedToast()
            Task { await self.refresh() }
        }
    }

    // MARK: - Actions

    func selectWallet(id: String) {
        guard let wallet = wallets.first(where: { $0.id.id == id }) else { return }
        selectedWallet = wallet
        session = session.onSelectWallet(walletId: wallet.id.id)
        Task { await refresh() }
    }

    func refresh() async {
        let result = await service.refresh(walletId: selectedWallet.id.id)
        session = session.onResult(result: result)
    }

    func onTaskOnce() async {
        await refresh()

        switch incomingReferralCode(code: activateCode, wallets: wallets.map { $0.toGem() }) {
        case let .activate(code): await useReferralCode(code)
        case let .confirm(code): isPresentingSheet = .activateCode(code: code)
        case .none: break
        }
    }

    func activatePendingReferral() async {
        guard let code = pendingReferral?.code else { return }
        await useReferralCode(code)
    }

    private func useReferralCode(_ code: String) async {
        do {
            let rewards = try await service.useReferralCode(wallet: selectedWallet, code: code)
            session = session.onRewards(rewards: rewards)
            showActivatedToast()
        } catch let error as GemServiceError {
            showError(error.text().text)
        } catch {
            debugLog("rewards error: \(error)")
        }
    }

    func onSelectRedemption(_ option: RewardRedemptionOptionViewModel) {
        if option.canRedeem {
            showRedemptionAlert(for: option.redemption)
        } else {
            showError(Localized.Rewards.insufficientPoints)
        }
    }

    func showRedemptionAlert(for redemption: GemRewardsRedemption) {
        let viewModel = RewardRedemptionOptionViewModel(redemption: redemption)
        isPresentingAlert = AlertMessage(
            title: viewModel.confirmationMessage,
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
        do {
            _ = try await service.redeem(wallet: selectedWallet, redemptionId: redemptionId)
            toastMessage = ToastMessage.success(Localized.Common.done)
        } catch let error as GemServiceError {
            showError(error.text().text)
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
