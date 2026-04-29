// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public struct ServiceStatusScene: View {
    @State private var model: ServiceStatusViewModel

    public init(model: ServiceStatusViewModel = ServiceStatusViewModel()) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            Section {
                ForEach(model.connectivityModels) { item in
                    ListItemView(
                        title: item.title,
                        titleTag: item.titleTag,
                        titleTagStyle: item.titleTagStyle,
                        titleTagType: item.titleTagType,
                        titleExtra: item.subtitle,
                    )
                }
            }
        }
        .listRowInsets(.assetListRowInsets)
        .listSectionSpacing(.compact)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .refreshable {
            await model.fetch()
        }
        .taskOnce {
            Task { await model.fetch() }
        }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
    }
}
