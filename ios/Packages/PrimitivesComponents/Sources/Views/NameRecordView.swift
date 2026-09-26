// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct NameRecordView: View {
    let model: NameRecordViewModel

    public init(model: NameRecordViewModel) {
        self.model = model
    }

    public var body: some View {
        VStack(alignment: .center, spacing: .zero) {
            switch model.state.indicator() {
            case .loading: LoadingView()
            case let .some(indicator): indicator.image
            case .none: EmptyView()
            }
        }.frame(width: .space16, height: .space16)
    }
}
