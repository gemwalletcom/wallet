// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemSettingsRow
import Localization
import Primitives
import SwiftUI

public struct SettingsScene: View {
    @Environment(\.openURL) private var openURL

    @State private var model: SettingsViewModel
    @Binding private var isPresentingWallets: Bool
    @Binding private var isPresentingSupport: Bool

    public init(
        model: SettingsViewModel,
        isPresentingWallets: Binding<Bool>,
        isPresentingSupport: Binding<Bool>,
    ) {
        _model = State(initialValue: model)
        _isPresentingWallets = isPresentingWallets
        _isPresentingSupport = isPresentingSupport
    }

    public var body: some View {
        List {
            ForEach(Array(model.sections.enumerated()), id: \.offset) { _, section in
                Section {
                    ForEach(section.rows, id: \.self) { row in
                        content(for: row)
                    }
                }
            }
            .listRowInsets(.assetListRowInsets)
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
    private func content(for row: GemSettingsRow) -> some View {
        switch row {
        case .wallets:
            NavigationCustomLink(
                with: ListItemView(
                    title: row.title,
                    subtitle: model.walletsValue,
                    imageStyle: .settings(assetImage: row.assetImage),
                ),
                action: onOpenWallets,
            )
        case .security:
            link(row, to: Scenes.Security())
        case .notifications:
            link(row, to: Scenes.Notifications())
        case .preferences:
            link(row, to: Scenes.Preferences())
        case .walletConnect:
            link(row, to: Scenes.WalletConnect())
        case .support:
            NavigationCustomLink(
                with: ListItemView(title: row.title, imageStyle: .settings(assetImage: row.assetImage)),
                action: onOpenSupport,
            )
        case .rewards:
            link(row, to: Scenes.Referral())
        case .aboutUs:
            link(row, to: Scenes.AboutUs())
        case .developer:
            link(row, to: Scenes.Developer())
        }
    }

    private func link(_ row: GemSettingsRow, to scene: some Hashable) -> some View {
        NavigationLink(value: scene) {
            ListItemView(title: row.title, imageStyle: .settings(assetImage: row.assetImage))
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
