// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemRewardsRedemption
import struct Gemstone.RewardRedemptionOption
import protocol Gemstone.GemRewardsServiceProtocol
import struct Gemstone.GemWalletRow
import func Gemstone.walletRow
import func Gemstone.walletRows
import struct Gemstone.GemRewardsState
import GemstonePrimitives
import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Style

@Observable
@MainActor
public final class RewardsViewModel: Sendable {
    private static let dateFormatter: DateComponentsFormatter = {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day, .hour, .minute]
        formatter.zeroFormattingBehavior = .dropLeading
        formatter.unitsStyle = .full
        return formatter
    }()

    private let service: any GemRewardsServiceProtocol
    private let activateCode: String?
    private let emptyState: GemRewardsState

    private(set) var selectedWallet: Wallet
    private(set) var wallets: [Wallet]

    var state: StateViewType<GemRewardsState> = .loading
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
        emptyState = service.state(rewards: nil)
        selectedWallet = wallet
        self.wallets = service.wallets(wallets: core).map { $0.toPrimitives() }
        self.activateCode = activateCode
    }

    // MARK: - UI Properties

    var title: String {
        Localized.Rewards.title
    }

    var referralCountTitle: String {
        Localized.Rewards.referrals
    }

    var pointsTitle: String {
        Localized.Rewards.points
    }

    var errorTitle: String {
        Localized.Errors.errorOccurred
    }

    var invitedByTitle: String {
        Localized.Rewards.invitedBy
    }

    var createCodeButtonTitle: String {
        Localized.Common.getStarted
    }

    var myReferralCodeTitle: String {
        Localized.Rewards.myReferralCode
    }

    var createCodeTitle: String {
        Localized.Rewards.InviteFriends.title
    }

    var createCodeDescription: String {
        Localized.Rewards.InviteFriends.description(String(rewardsState.inviteRewardPoints).boldMarkdown())
    }

    var activateCodeFooterTitle: String {
        Localized.Rewards.ActivateReferralCode.title
    }

    var activateCodeFooterDescription: String {
        Localized.Rewards.ActivateReferralCode.description
    }

    var statsSectionTitle: String {
        Localized.Common.info
    }

    var showsWalletSelector: Bool {
        wallets.count > 1
    }

    var walletSelectorModel: SelectWalletViewModel {
        SelectWalletViewModel(
            rows: walletRows(wallets: wallets.map { $0.toGem() }),
            selectedRow: selectedWalletRow,
        )
    }

    var shareText: String? {
        referralLink.map { Localized.Rewards.shareText($0) }
    }

    var referralLink: String? {
        rewardsState.referralLink
    }

    var redemptions: [GemRewardsRedemption] {
        rewardsState.redemptions
    }

    var rewardsState: GemRewardsState {
        if case let .data(state) = state {
            return state
        }
        return emptyState
    }

    var referralCode: String? {
        rewardsState.referralCode
    }

    var referralCodeListItem: ListItemModel? {
        referralCode.map { ListItemModel(title: myReferralCodeTitle, subtitle: $0) }
    }

    var referralCountListItem: ListItemModel {
        ListItemModel(title: referralCountTitle, subtitle: referralCountText)
    }

    var pointsListItem: ListItemModel {
        ListItemModel(title: pointsTitle, subtitle: pointsText)
    }

    var invitedByListItem: ListItemModel? {
        invitedBy.map { ListItemModel(title: invitedByTitle, subtitle: $0) }
    }

    var referralCountText: String {
        rewardsState.referralCountText
    }

    var pointsText: String {
        rewardsState.pointsText
    }

    var invitedBy: String? {
        rewardsState.usedReferralCode
    }

    var unverifiedTitle: String {
        Localized.Rewards.Unverified.title
    }

    var unverifiedDescription: String {
        Localized.Rewards.Unverified.description
    }

    var disableReason: String? {
        rewardsState.disableReason
    }

    var pendingVerificationAfter: Date? {
        rewardsState.verifyAfter
    }

    var pendingReferralTitle: String {
        Localized.Rewards.Pending.title
    }

    var pendingReferralDescription: String? {
        guard let pendingDate = pendingVerificationAfter else { return nil }
        if rewardsState.canActivatePendingReferral {
            return Localized.Rewards.Pending.descriptionReady
        }
        guard let timeString = Self.dateFormatter.string(from: .now, to: pendingDate) else { return nil }
        return Localized.Rewards.Pending.description(timeString)
    }

    var pendingReferralButtonTitle: String {
        Localized.Transfer.confirm
    }

    var activatePendingButtonType: ButtonType {
        rewardsState.canActivatePendingReferral ? .primary() : .primary(.disabled)
    }

    var selectedWalletRow: GemWalletRow {
        walletRow(wallet: selectedWallet.toGem())
    }

    var walletBarViewModel: WalletBarViewViewModel {
        WalletBarViewViewModel(name: selectedWalletRow.name, image: selectedWalletRow.avatarImage)
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
            state = .data(service.state(rewards: rewards))
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
            Task { await self.load() }
        }
    }

    // MARK: - Actions

    func selectWallet(id: String) {
        guard let wallet = wallets.first(where: { $0.id.id == id }) else { return }
        selectedWallet = wallet
        Task { await load(wallet: wallet) }
    }

    func load() async {
        await load(wallet: selectedWallet)
    }

    func onTaskOnce() async {
        await load()

        if wallets.count == 1, activateCode != nil {
            await useReferralCode()
        } else if let code = activateCode {
            isPresentingSheet = .activateCode(code: code)
        }
    }

    private func useReferralCode() async {
        guard let code = activateCode else { return }
        do {
            try await service.useReferralCode(wallet: selectedWallet, code: code)
            showActivatedToast()
            await load()
        } catch {
            showError(error.localizedDescription)
        }
    }

    func activatePendingReferral() async {
        guard let code = rewardsState.usedReferralCode else { return }
        do {
            try await service.useReferralCode(wallet: selectedWallet, code: code)
            showActivatedToast()
            await load()
        } catch {
            showError(error.localizedDescription)
        }
    }

    func showRedemptionAlert(for redemption: GemRewardsRedemption) {
        let viewModel = RewardRedemptionOptionViewModel(redemption: redemption)
        let option = redemption.option
        isPresentingAlert = AlertMessage(
            title: viewModel.confirmationMessage,
            message: "",
            actions: [
                AlertAction(title: Localized.Transfer.confirm, isDefaultAction: true) { [weak self] in
                    Task {
                        await self?.redeem(option: option)
                        await self?.load()
                    }
                },
                .cancel(title: Localized.Common.cancel),
            ],
        )
    }

    func redeem(option: RewardRedemptionOption) async {
        do {
            _ = try await service.redeem(wallet: selectedWallet, redemptionId: option.id)
            toastMessage = ToastMessage.success(Localized.Common.done)
        } catch {
            showError(error.localizedDescription)
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

    private func load(wallet: Wallet) async {
        state = .loading
        do {
            let rewards = try await service.getRewards(wallet: wallet)
            state = .data(service.state(rewards: rewards))
        } catch {
            state = .noData
        }
    }
}
