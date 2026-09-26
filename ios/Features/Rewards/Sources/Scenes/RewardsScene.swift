// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemRewardsRedemption
import enum Gemstone.GemServiceError
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct RewardsScene: View {
    @State private var model: RewardsSceneViewModel

    public init(model: RewardsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            switch model.viewState.state {
            case .loading:
                CenterLoadingView()
            case let .error(error):
                stateErrorView(error: error)
            case .data, .noData:
                inviteFriendsSection
                if let notice = model.rewardsState.errorNotice {
                    Section {
                        GemListRowView(row: notice)
                    }
                }
                statusSection
                infoSections
                if model.redemptionOptions.isNotEmpty {
                    redemptionOptionsSection(options: model.redemptionOptions)
                }
            }
        }
        .refreshable { await model.refresh() }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .listStyle(.insetGrouped)
        .navigationTitle(model.title)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                if model.showsWalletSelector {
                    WalletBarView(row: model.selectedWalletRow) {
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
                        ListItemView(model: wallet.nameListItem)
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

    private func stateErrorView(error: GemServiceError) -> some View {
        Section {
            StateEmptyView(
                title: model.errorTitle,
                description: error.text().text,
                image: nil,
            ) {
                Button(Localized.Common.tryAgain) {
                    Task { await model.refresh() }
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

                if model.action(.share) {
                    Button {
                        model.isPresentingSheet = .share
                    } label: {
                        HStack(spacing: Spacing.small) {
                            Images.System.share
                            Text(Localized.Rewards.InviteFriends.title)
                        }
                    }
                    .buttonStyle(.blue())
                } else if model.action(.createCode) {
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

        if model.action(.useReferralCode) {
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

    private func redemptionOptionsSection(options: [GemRewardsRedemption]) -> some View {
        Section {
            ForEach(options, id: \.id) { option in
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

    private var infoSections: some View {
        ForEach(model.sections) { section in
            Section {
                ForEach(section.values) { item in
                    if case let .text(title, _) = item.row, title == .myReferralCode {
                        GemListRowView(row: item.row)
                            .contextMenu(model.referralLink.map { [.copy(value: $0)] } ?? [])
                    } else {
                        GemListRowView(row: item.row)
                    }
                }
            } header: {
                if let title = section.title {
                    Text(title)
                }
            }
        }
    }

    @ViewBuilder
    private var statusSection: some View {
        if let notice = model.rewardsState.statusNotice {
            Section {
                GemListRowView(row: notice)

                if model.pendingReferral != nil {
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
