// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRewardsServiceProtocol
import struct Gemstone.GemRewardsState
import GemstonePrimitives
import GemstoneServices
import Components
import Foundation
import Localization
import Preferences
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

    private(set) var selectedWallet: Wallet
    private(set) var wallets: [Wallet]

    var state: StateViewType<Rewards> = .loading
    var toastMessage: ToastMessage?
    var isPresentingSheet: RewardsSheetType?
    var isPresentingAlert: AlertMessage?

    public init?(service: any GemRewardsServiceProtocol, wallets: [Wallet], currentWallet: Wallet?, activateCode: String? = nil) {
        let core = wallets.map { $0.map() }
        guard let wallet = service.selectedWallet(current: currentWallet?.map(), wallets: core).map({ $0.map() }) else { return nil }
        self.service = service
        selectedWallet = wallet
        self.wallets = service.wallets(wallets: core).map { $0.map() }
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
        Localized.Rewards.InviteFriends.description(String(100).boldMarkdown())
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
        SelectWalletViewModel(wallets: wallets, selectedWallet: selectedWallet)
    }

    var rewards: Rewards? {
        if case let .data(rewards) = state {
            return rewards
        }
        return nil
    }

    var shareText: String? {
        guard let code = rewards?.code else { return nil }
        let link = (try? service.referralLink(code: code).absoluteString) ?? ""
        return Localized.Rewards.shareText(link)
    }

    var referralLink: String? {
        guard let code = rewards?.code else { return nil }
        return (try? service.referralLink(code: code).absoluteString) ?? ""
    }

    var rewardsState: GemRewardsState {
        service.state(rewards: rewards)
    }

    var unverifiedTitle: String {
        Localized.Rewards.Unverified.title
    }

    var unverifiedDescription: String {
        Localized.Rewards.Unverified.description
    }

    var disableReason: String? {
        rewards?.disableReason
    }

    var pendingVerificationAfter: Date? {
        rewards?.verifyAfter
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

    var walletBarViewModel: WalletBarViewViewModel {
        let walletVM = WalletViewModel(wallet: selectedWallet)
        return WalletBarViewViewModel(name: walletVM.name, image: walletVM.avatarImage)
    }

    var rewardsUrl: URL {
        AppUrl.rewards(.rewards)
    }

    var createCodeViewModel: CreateRewardsCodeViewModel {
        CreateRewardsCodeViewModel(
            service: service,
            wallet: selectedWallet,
        ) { [weak self] rewards in
            self?.state = .data(rewards)
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

    func selectWallet(_ wallet: Wallet) {
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
        guard let code = rewards?.usedReferralCode else { return }
        do {
            try await service.useReferralCode(wallet: selectedWallet, code: code)
            showActivatedToast()
            await load()
        } catch {
            showError(error.localizedDescription)
        }
    }

    func canRedeem(option: RewardRedemptionOption) -> Bool {
        guard let rewards else { return false }
        return rewards.points >= option.points
    }

    func showRedemptionAlert(for option: RewardRedemptionOption) {
        let viewModel = RewardRedemptionOptionViewModel(option: option)
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
            state = .data(rewards)
        } catch {
            state = .noData
        }
    }
}
