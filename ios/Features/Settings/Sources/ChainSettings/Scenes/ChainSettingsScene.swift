// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import PrimitivesComponents
import Style
import SwiftUI

public struct ChainSettingsScene: View {
    @State private var model: ChainSettingsSceneViewModel

    public init(model: ChainSettingsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            ForEach(model.sections) { section in
                Section(section.title) {
                    content(for: section)
                }
            }
        }
        .refreshable {
            await model.load()
        }
        .alert(
            model.deleteConfirmationTitle(for: model.nodeDelete?.host ?? ""),
            presenting: $model.nodeDelete,
            sensoryFeedback: .warning,
            actions: { _ in
                Button(
                    model.deleteButtonTitle,
                    role: .destructive,
                    action: model.onDeleteNode,
                )
            },
        )
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button(action: model.onPresentImportNode) {
                    Images.System.plus
                        .font(.body.weight(.semibold))
                }
            }
        }
        .sheet(isPresented: $model.isPresentingImportNode) {
            NavigationStack {
                AddNodeScene(model: model.addNodeModel(), onDismiss: model.onDismissImportNode)
            }
        }
        .alertSheet($model.isPresentingAlertMessage)
        .navigationTitle(model.title)
        .listSectionSpacing(.compact)
        .taskOnce {
            Task { await model.load() }
        }
    }

    @ViewBuilder
    private func content(for section: ChainSettingsSectionViewModel) -> some View {
        switch section.kind {
        case .nodes:
            ForEach(model.nodesModels) { nodeModel in
                ListItemSelectionView(
                    title: nodeModel.title,
                    titleExtra: nodeModel.titleExtra,
                    titleTag: nodeModel.titleTag,
                    titleTagType: nodeModel.titleTagType,
                    titleTagStyle: nodeModel.titleTagStyle,
                    subtitle: .none,
                    subtitleExtra: .none,
                    value: nodeModel.url,
                    selection: nodeModel.selection,
                    action: model.onSelectNode,
                )
                .contextMenu(
                    .copy(value: nodeModel.url),
                )
                .if(nodeModel.canDelete) {
                    $0.swipeActions(edge: .trailing) {
                        Button(model.deleteButtonTitle, role: .destructive) {
                            model.onSelectNodeForDeletion(nodeModel.node)
                        }
                        .tint(Colors.red)
                    }
                }
            }
        case .explorer:
            ForEach(model.explorers, id: \.name) { explorer in
                ListItemSelectionView(
                    title: explorer.name,
                    titleExtra: .none,
                    titleTag: .none,
                    titleTagType: .none,
                    subtitle: .none,
                    subtitleExtra: .none,
                    value: explorer.name,
                    selection: explorer.isSelected ? explorer.name : .none,
                    action: model.onSelectExplorer(name:),
                )
            }
        }
    }
}
