// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
                        AssetStatusView(model: VerificationStatusViewModel(status: status.toPrimitives()), action: model.onSelectStatus)
                    }
                case let .info(rows):
                    Section {
                        ForEach(model.infoRows(rows), content: infoRowView)
                    }
                case let .attributes(attributes):
                    Section(Localized.Nft.properties) {
                        ForEach(attributes, id: \.self) {
                            ListItemView(model: model.attributeListItem($0))
                        }
                    }
                case let .links(links):
                    Section(Localized.Social.links) {
                        SocialLinksView(model: model.socialLinksModel(links))
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
    private func infoRowView(_ row: CollectibleInfoRowModel) -> some View {
        if let assetImage = row.assetImage {
            ListItemImageView(title: row.title, subtitle: row.subtitle, assetImage: assetImage)
        } else if let copyValue = row.copyValue, let explorer = row.explorer {
            ListItemView(model: row.listItem)
                .explorerContext(ExplorerContextData(copyValue: copyValue, explorerLink: explorer))
        } else if let copyValue = row.copyValue {
            ListItemView(model: row.listItem)
                .contextMenu(.copy(value: copyValue.rawValue, onCopy: model.onSelectCopyValue))
        } else {
            ListItemView(model: row.listItem)
        }
    }
}
