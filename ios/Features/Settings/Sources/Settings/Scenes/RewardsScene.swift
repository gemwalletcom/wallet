// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct RewardsScene: View {
    @State private var model: RewardsViewModel

    public init(model: RewardsViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            switch model.state {
            case .loading:
                CenterLoadingView()
            case let .error(error):
                stateErrorView(error: error)
            case .data:
                inviteFriendsSection
                if let notice = model.rewardsState.errorNotice {
                    Section {
                        GemListRowView(row: notice)
                    }
                }
                statusSection
                if model.rewardsState.showsInfo {
                    infoSection
                }
                if model.redemptionOptions.isNotEmpty {
                    redemptionOptionsSection(options: model.redemptionOptions)
                }
            case .noData:
                inviteFriendsSection
            }
        }
        .refreshable { await model.load() }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .listStyle(.insetGrouped)
        .navigationTitle(model.title)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                if model.showsWalletSelector {
                    WalletBarView(model: model.walletBarViewModel) {
                        model.isPresentingSheet = .walletSelector
                    }
                } else {
                    Button {
                        model.isPresentingSheet = .url(model.rewardsUrl)
                    } label: {
                        Images.System.info
                    }
                }
            }
        }
        .sheet(item: $model.isPresentingSheet) { sheet in
            switch sheet {
            case .walletSelector:
                SelectableListNavigationStack(
                    model: model.walletSelectorModel,
                    onFinishSelection: { rows in
                        if let row = rows.first {
                            model.selectWallet(id: row.id)
                        }
                        model.isPresentingSheet = nil
                    },
                    listContent: { wallet in
                        SimpleListItemView(model: wallet)
                    },
                )
            case .share:
                if let shareText = model.shareText {
                    ShareSheet(activityItems: [shareText])
                }
            case .createCode:
                TextInputScene(model: model.createCodeViewModel) {
                    model.isPresentingSheet = nil
                }
                .presentationDetents([.medium])
            case let .activateCode(code):
                TextInputScene(model: model.redeemCodeViewModel(code: code)) {
                    model.isPresentingSheet = nil
                }
                .presentationDetents([.medium])
            case let .url(url):
                SFSafariView(url: url)
            }
        }
        .taskOnce {
            Task { await model.onTaskOnce() }
        }
        .toast(message: $model.toastMessage)
        .alertSheet($model.isPresentingAlert)
    }

    private func stateErrorView(error: Error) -> some View {
        Section {
            StateEmptyView(
                title: model.errorTitle,
                description: error.localizedDescription,
                image: nil,
            ) {
                Button(Localized.Common.tryAgain) {
                    Task { await model.load() }
                }
                .buttonStyle(.blue())
            }
        }
    }

    @ViewBuilder
    private var inviteFriendsSection: some View {
        Section {
            VStack(spacing: Spacing.large) {
                Text("🎁")
                    .font(.app.extraLargeTitle)
                    .padding(.top, Spacing.medium)

                VStack(spacing: Spacing.small) {
                    Text(model.createCodeTitle)
                        .font(.title2.bold())
                        .multilineTextAlignment(.center)

                    Text(.init(model.createCodeDescription))
                        .textStyle(.calloutSecondary)
                        .multilineTextAlignment(.center)
                }

                HStack(spacing: Spacing.medium) {
                    featureItem(emoji: "👥", text: Localized.Rewards.InviteFriends.title)
                    featureItem(emoji: "💎", text: Localized.Rewards.EarnPoints.title)
                    featureItem(emoji: "🎉", text: Localized.Rewards.GetRewards.title)
                }

                if model.rewardsState.canInvite {
                    Button {
                        model.isPresentingSheet = .share
                    } label: {
                        HStack(spacing: Spacing.small) {
                            Images.System.share
                            Text(Localized.Rewards.InviteFriends.title)
                        }
                    }
                    .buttonStyle(.blue())
                } else if !model.rewardsState.hasReferralCode {
                    Button {
                        model.isPresentingSheet = .createCode
                    } label: {
                        Text(model.createCodeButtonTitle)
                    }
                    .buttonStyle(.blue())
                }
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, Spacing.small)
        }

        if model.rewardsState.canUseReferralCode {
            Section {
                Button {
                    model.isPresentingSheet = .activateCode(code: "")
                } label: {
                    Text(model.activateCodeFooterTitle)
                        .frame(maxWidth: .infinity)
                }
            } footer: {
                Text(model.activateCodeFooterDescription)
            }
        }
    }

    private func featureItem(emoji: String, text: String) -> some View {
        VStack(spacing: Spacing.extraSmall) {
            Text(emoji)
                .font(.title2)
            Text(text)
                .font(.caption)
                .foregroundStyle(Colors.secondaryText)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
    }

    private func redemptionOptionsSection(options: [RewardRedemptionOptionViewModel]) -> some View {
        Section {
            ForEach(options) { option in
                NavigationCustomLink(
                    with: ListItemView(model: option.listItem),
                ) {
                    model.onSelectRedemption(option)
                }
            }
        } header: {
            Text(Localized.Rewards.WaysSpend.title)
        }
    }

    private var infoSection: some View {
        Section {
            ForEach(Array(model.infoRows.enumerated()), id: \.offset) { _, row in
                if case let .text(title, _) = row, title == .myReferralCode {
                    GemListRowView(row: row)
                        .contextMenu(model.referralLink.map { [.copy(value: $0)] } ?? [])
                } else {
                    GemListRowView(row: row)
                }
            }
        } header: {
            Text(model.statsSectionTitle)
        }
    }

    @ViewBuilder
    private var statusSection: some View {
        if let notice = model.rewardsState.statusNotice {
            Section {
                GemListRowView(row: notice)

                if model.rewardsState.showsPendingActivation {
                    HStack {
                        Spacer()
                        StateButton(
                            text: model.pendingReferralButtonTitle,
                            type: model.activatePendingButtonType,
                        ) {
                            Task { await model.activatePendingReferral() }
                        }
                        .frame(height: .scene.button.height)
                        .frame(maxWidth: .scene.button.maxWidth)
                        Spacer()
                    }
                }
            }
        }
    }
}
