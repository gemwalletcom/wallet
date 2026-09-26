// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

public struct SettingsScene: View {
    @Environment(\.openURL) private var openURL

    @State private var model: SettingsSceneViewModel
    @Binding private var isPresentingWallets: Bool
    @Binding private var isPresentingSupport: Bool

    public init(
        model: SettingsSceneViewModel,
        isPresentingWallets: Binding<Bool>,
        isPresentingSupport: Binding<Bool>,
    ) {
        _model = State(initialValue: model)
        _isPresentingWallets = isPresentingWallets
        _isPresentingSupport = isPresentingSupport
    }

    public var body: some View {
        ListSectionView(sections: model.sections) { row in
            content(for: row)
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listStyle(.insetGrouped)
        .listSectionSpacing(.compact)
        .bindQuery(model.walletsQuery)
        .navigationTitle(model.title)
    }
}

// MARK: - UI Components

extension SettingsScene {
    @ViewBuilder
    private func content(for row: GemListRow) -> some View {
        switch row.tap() {
        case .wallets:
            NavigationCustomLink(with: GemListRowView(row: row), action: onOpenWallets)
        case .security:
            link(row, to: Scenes.Security())
        case .notifications:
            link(row, to: Scenes.Notifications())
        case .preferences:
            link(row, to: Scenes.Preferences())
        case .walletConnect:
            link(row, to: Scenes.WalletConnect())
        case .support:
            NavigationCustomLink(with: GemListRowView(row: row), action: onOpenSupport)
        case .rewards:
            link(row, to: Scenes.Rewards())
        case .aboutUs:
            link(row, to: Scenes.AboutUs())
        case .developer:
            link(row, to: Scenes.Developer())
        default:
            GemListRowView(row: row)
        }
    }

    private func link(_ row: GemListRow, to scene: some Hashable) -> some View {
        NavigationLink(value: scene) {
            GemListRowView(row: row)
        }
    }
}

// MARK: - Actions

extension SettingsScene {
    private func onOpenWallets() {
        isPresentingWallets = true
    }

    private func onOpenSupport() {
        isPresentingSupport = true
    }
}
