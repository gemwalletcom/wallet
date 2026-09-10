// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemCollectibleIdentifier
import enum Gemstone.GemCollectibleRow
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct CollectibleScene: View {
    @State private var model: CollectibleViewModel

    public init(model: CollectibleViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            headerSectionView
            ForEach(model.sections, id: \.self) { section in
                switch section {
                case let .status(status):
                    Section {
                        AssetStatusView(model: VerificationStatusViewModel(status: status.map()), action: model.onSelectStatus)
                    }
                case let .info(rows):
                    Section {
                        ForEach(rows, id: \.self, content: infoRowView)
                    }
                case let .attributes(attributes):
                    Section(Localized.Nft.properties) {
                        ForEach(attributes, id: \.self) {
                            ListItemView(title: $0.name, subtitle: model.attributeText($0.value))
                        }
                    }
                case let .links(links):
                    Section(Localized.Social.links) {
                        SocialLinksView(model: SocialLinksViewModel(assetLinks: links.map { $0.map() }))
                    }
                }
            }
        }
        .environment(\.defaultMinListHeaderHeight, 0)
        .listSectionSpacing(.compact)
        .contentMargins([.top], .small, for: .scrollContent)
        .navigationTitle(model.title)
        .toolbar {
            ToolbarItem(placement: .principal) {
                HStack(spacing: .tiny) {
                    Text(model.title)
                        .font(.headline)
                    if model.isVerified {
                        VerifiedBadgeView(font: .subheadline)
                    }
                }
            }
        }
        .alertSheet($model.isPresentingAlertMessage)
        .toast(message: $model.isPresentingToast)
        .sheet(isPresented: $model.isPresentingReportSheet) {
            ReportNavigationStack(model: model.reportModel())
        }
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(model: InfoSheetModelFactory.create(from: $0))
        }
        .bindQuery(model.query)
    }
}

// MARK: - UI

extension CollectibleScene {
    private var headerSectionView: some View {
        Section {
            NftImageView(
                assetImage: model.assetImage,
                isImageLoaded: $model.isImageLoaded,
            )
            .aspectRatio(1, contentMode: .fill)
        } header: {
            Spacer()
        } footer: {
            HeaderButtonsView(buttons: model.headerButtons, action: model.onSelectHeaderButton(type:))
                .padding(.top, .medium)
                .padding(.bottom, .small)
        }
        .frame(maxWidth: .infinity)
        .textCase(nil)
        .listRowSeparator(.hidden)
        .listRowInsets(EdgeInsets())
        .contextMenu(model.imageContextMenuItems)
    }

    @ViewBuilder
    private func infoRowView(_ row: GemCollectibleRow) -> some View {
        switch row {
        case let .collection(name):
            ListItemView(title: Localized.Nft.collection, subtitle: name)
        case let .network(chain):
            let chain = Primitives.Chain(core: chain)
            ListItemImageView(
                title: Localized.Transfer.network,
                subtitle: chain.networkName,
                assetImage: model.networkImage(chain: chain),
            )
        case let .contract(identifier):
            identifierRowView(
                title: Localized.Asset.contract,
                identifier: identifier,
                copyValue: .address(value: identifier.value, chain: model.assetData.asset.chain),
            )
        case let .tokenId(identifier):
            identifierRowView(title: Localized.Asset.tokenId, identifier: identifier, copyValue: .plain(identifier.value))
        }
    }

    @ViewBuilder
    private func identifierRowView(title: String, identifier: GemCollectibleIdentifier, copyValue: CopyValue) -> some View {
        if let explorer = identifier.explorer {
            ListItemView(title: title, subtitle: identifier.text)
                .explorerContext(ExplorerContextData(copyValue: copyValue, explorerLink: explorer.map()))
        } else {
            ListItemView(title: title, subtitle: identifier.text)
                .contextMenu(.copy(value: identifier.value, onCopy: model.onSelectCopyValue))
        }
    }
}
