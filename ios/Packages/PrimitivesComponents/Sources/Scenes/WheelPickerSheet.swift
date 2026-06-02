// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public struct WheelPickerSheet<T: WheelPickerDisplayable>: View {
    private let title: String
    private let options: [T]
    @Binding private var selection: T

    public init(title: String, options: [T], selection: Binding<T>) {
        self.title = title
        self.options = options
        _selection = selection
    }

    public var body: some View {
        NavigationStack {
            WheelPickerView(options: options, selection: $selection)
                .navigationBarTitleDisplayMode(.inline)
                .navigationTitle(title)
                .toolbar {
                    ToolbarDismissItem(
                        type: .close,
                        placement: .topBarLeading,
                    )
                }
        }
        .presentationDetents([.height(300)])
    }
}
