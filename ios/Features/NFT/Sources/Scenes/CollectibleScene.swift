// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemCollectibleDetails
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct CollectibleScene: View {
    @State private var model: CollectibleSceneViewModel

    public init(model: CollectibleSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            switch model.state {
            case .loading, .noData:
                CenterLoadingView()
            case let .error(error):
                stateErrorView(error: error)
            case let .data(details):
                content(details)
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
                    if model.details?.isVerified == true {
                        VerifiedBadgeView(font: .subheadline)
                    }
                }
            }
        }
        .alertSheet($model.isPresentingAlertMessage)
        .toast(message: $model.isPresentingToast)
        .sheet(isPresented: $model.isPresentingReportSheet) {
            if let assetData = model.assetData {
                ReportNavigationStack(model: model.reportModel(assetData))
            }
        }
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(sheet: $0)
        }
        .bindQuery(model.query)
        .taskOnce {
            Task { await model.load() }
        }
    }
}

// MARK: - UI

extension CollectibleScene {
    @ViewBuilder
    private func content(_ assetDetails: NFTAssetDetails) -> some View {
        let details = model.details(assetDetails)
        headerSectionView(assetDetails.assetData, details: details)
        ForEach(details.sections, id: \.self) { group in
            switch group.section {
            case let .status(status):
                Section {
                    AssetStatusView(status: status.toPrimitives(), action: model.onSelectStatus)
                }
            case let .info(rows):
                Section {
                    ForEach(rows, id: \.self) { row in
                        GemListRowView(row: row, onSelectAddress: model.onSelectContract)
                    }
                }
            case let .attributes(attributes):
                Section(group.title.text ?? .empty) {
                    ForEach(attributes, id: \.self) {
                        ListItemView(model: model.attributeListItem($0))
                    }
                }
            case let .links(links):
                Section(group.title.text ?? .empty) {
                    SocialLinksView(links: links)
                }
            }
        }
    }

    private func headerSectionView(_ assetData: NFTAssetData, details: GemCollectibleDetails) -> some View {
        Section {
            NftImageView(
                assetImage: model.assetImage(assetData),
                isImageLoaded: $model.isImageLoaded,
            )
            .aspectRatio(1, contentMode: .fill)
        } header: {
            Spacer()
        } footer: {
            HeaderButtonsView(buttons: details.header.headerButtons, menuTitle: model.title, menuItems: model.menuItems(details), action: model.onSelectHeaderButton)
                .padding(.top, .medium)
                .padding(.bottom, .small)
        }
        .frame(maxWidth: .infinity)
        .textCase(nil)
        .listRowSeparator(.hidden)
        .listRowInsets(EdgeInsets())
        .contextMenu(model.imageContextMenuItems(details))
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
}
