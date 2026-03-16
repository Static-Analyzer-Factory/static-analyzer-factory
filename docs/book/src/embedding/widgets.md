# SAF Widgets

SAF provides embeddable widgets that bring interactive program analysis into any
web page. Widgets are implemented as iframes pointing to the SAF playground in
embed mode.

## Basic Usage

Add a widget to any HTML page:

```html
<div class="saf-widget">
  <iframe
    src="../../playground/?embed=true&split=true&example=taint_flow&graph=cfg"
    loading="lazy"
  ></iframe>
</div>
```

## URL Parameters

| Parameter | Values | Description |
|-----------|--------|-------------|
| `embed` | `true` | Enables embed mode (hides toolbar, navigation) |
| `split` | `true` | Shows both source editor and graph side by side |
| `example` | `taint_flow`, `pointer_alias`, `complex_cfg`, `indirect_call`, etc. | Pre-loaded example slug |
| `graph` | `cfg`, `callgraph`, `defuse`, `valueflow`, `pta` | Initial graph to display |

## Styling

Add the SAF widget styles to your CSS:

```css
.saf-widget {
  border: 1px solid #333;
  border-radius: 8px;
  overflow: hidden;
  margin: 1em 0;
}

.saf-widget iframe {
  width: 100%;
  height: 400px;
  border: none;
}
```

Adjust the `height` to fit your content. For split mode with source + graph,
`500px` or more is recommended.

## mdBook Integration

In mdBook pages, use raw HTML:

```html
<div class="saf-widget">
  <iframe
    src="../../playground/?embed=true&split=true&example=taint_flow&graph=valueflow"
    loading="lazy"
  ></iframe>
</div>
```

The `custom.css` file included in this book already provides the widget styles.

## Examples

### CFG Visualization

```html
<div class="saf-widget">
  <iframe
    src="../../playground/?embed=true&split=true&example=complex_cfg&graph=cfg"
    loading="lazy"
  ></iframe>
</div>
```

### Call Graph with Indirect Calls

```html
<div class="saf-widget">
  <iframe
    src="../../playground/?embed=true&split=true&example=indirect_call&graph=callgraph"
    loading="lazy"
  ></iframe>
</div>
```

### Points-To Analysis

```html
<div class="saf-widget">
  <iframe
    src="../../playground/?embed=true&split=true&example=pointer_alias&graph=pta"
    loading="lazy"
  ></iframe>
</div>
```

### Value Flow (Taint)

```html
<div class="saf-widget">
  <iframe
    src="../../playground/?embed=true&split=true&example=taint_flow&graph=valueflow"
    loading="lazy"
  ></iframe>
</div>
```

## Sizing Recommendations

| Mode | Recommended Height | Description |
|------|-------------------|-------------|
| Graph only (`embed=true`) | 400px | Shows graph visualization only |
| Split view (`embed=true&split=true`) | 500px+ | Shows source code + graph side by side |
| Full height | 600px+ | Best for complex graphs with many nodes |

## How It Works

When `embed=true` is set:

1. The playground hides the top navigation bar
2. The specified example is loaded automatically
3. The specified graph type is selected
4. With `split=true`, both the source editor and graph panel are visible

The widget runs entirely client-side via WebAssembly. No server calls are made
for the analysis itself (C compilation still uses Compiler Explorer if the
example contains C code).

## Responsive Design

Widgets adapt to their container width. For narrow containers, consider
increasing the height:

```css
@media (max-width: 768px) {
  .saf-widget iframe {
    height: 300px;
  }
}
```

## Security

Widgets use iframe sandboxing. The embedded playground cannot access the parent
page's DOM or cookies. Communication between the parent page and widget is not
currently supported.
