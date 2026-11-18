---
title: Viewport
order: 4
---

The viewport is a flexible area where you can arrange your views:
You can grab the title of any view to dock it to different parts of the viewport or to form tabs.

## View controls

<picture>
  <img src="https://static.rerun.io/view-controls/e911cec51fcf840e014340b3cb135b7faeb2e8b6/full.png" alt="">
</picture>


Clicking on the title of a view has the same effect as selecting it in the [blueprint panel](blueprints.md)
and will show additional information and settings in the [selection panel](selection.md).

For more information on how to navigate within a specific view, hover its help icon in the top right corner.

The maximize button makes a single view fill the entire viewport.
Only one view can be maximized at a time.


## View classes

Rerun includes multiple view classes, each dedicated to a specific type of visualization; for example, a 3D scene or a timeseries plot.
See the [views reference page](../types/views.md) for a list of available view classes.

The view class, which is specified upon creation, determines which entities it can display, how it displays them, and the way they can be interacted with.
Views can be created both from viewer and from code (see [Configure the Viewer](../../getting-started/configure-the-viewer.md)).

To learn more about the _internals_ of how view classes work, check the [guide on implementing custom views](../../howto/visualization/extend-ui.md).

## Image region inspection

Click **Region** in a 2D view's title bar, then drag a rectangle on an image.
The **Image region** window shows the crop, its width × height in image pixels, and the selected coordinate ranges.
The crop displays each pixel at five UI points; scroll within the crop to inspect a region larger than its preview window.
Coordinates start at the image's top-left corner; range ends are exclusive.
Selection rounds outward to whole pixels and clips to the image boundary, including when you drag backwards or finish outside the image.

Click **Region** again to resume normal dragging to pan.
Scroll and zoom keep the selection aligned with the same image pixels.
Use **Clear region** in the crop window to remove it, or press Escape over the image while Region mode is enabled.
Selections are temporary and separate for each view.
The crop follows the current image at the selected entity; hiding the image or changing its resolution clears the selection.

Inspection supports color, grayscale, depth, and segmentation images in a 2D plane, including in-plane transforms, using the image's current color mapping.
It does not support videos, image planes outside z = 0, region statistics, or crop export.
