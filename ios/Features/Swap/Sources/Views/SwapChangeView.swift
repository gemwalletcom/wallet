// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Style
import SwiftUI

struct SwapChangeView: View {
    @Binding private var fromId: AssetId?
    @Binding private var toId: AssetId?

    init(
        fromId: Binding<AssetId?> = .constant(.none),
        toId: Binding<AssetId?> = .constant(.none),
    ) {
        _fromId = fromId
        _toId = toId
    }

    var body: some View {
        Button {
            swap(&fromId, &toId)
        } label: {
            Images.System.arrowSwap
                .resizable()
                .renderingMode(.template)
                .scaledToFit()
                .frame(size: .medium)
                .foregroundStyle(Colors.gray)
                .padding(.small)
                .background(Colors.listStyleColor, in: Circle())
                .overlay(
                    Circle().strokeBorder(Colors.grayBackground, lineWidth: .space4),
                )
        }
    }
}
