bl_info = {
    'name': 'Bezier Export',
    'author': 'spazzylemons',
    'version': (1, 0, 0),
    'blender': (3, 4, 1),
    'location': 'View3D > Object',
    'description': 'Export Bezier Curve',
    'category': 'Import-Export',
}

import bpy
import bpy_extras
import math
import struct

class BezierExport(bpy.types.Operator, bpy_extras.io_utils.ExportHelper):
    """Export the selected Bezier curve."""
    bl_idname = 'bezier_export.bezier_export'
    bl_label = 'Export Bezier Curve'

    filename_ext = '.bin'

    def execute(self, context):
        try:
            # get the active curve
            curve = bpy.data.curves[context.active_object.data.name]
            # get the points
            points = curve.splines[0].bezier_points
            with open(self.filepath, 'wb') as file:
                # write number of points
                file.write(bytes([len(points)]))
                # helper func to write coordinate
                def write_co(co):
                    for scalar in co.xzy:
                        v = int(scalar * 256)
                        file.write(struct.pack('<h', v))
                # iterate over the points in the curve - export them as 8.8
                for point in points:
                    write_co(point.handle_left)
                    write_co(point.co)
                    write_co(point.handle_right)
                    file.write(bytes([int(256 * ((point.tilt % (2 * math.pi)) / (2 * math.pi)))]))
        except BaseException as e:
            self.report({'ERROR'}, repr(e))
            return {'CANCELLED'}
        else:
            return {'FINISHED'}

def func(self, context):
    self.layout.operator(BezierExport.bl_idname)

def register():
    bpy.utils.register_class(BezierExport)
    bpy.types.VIEW3D_MT_object.append(func)

def unregister():
    bpy.types.VIEW3D_MT_object.remove(func)
    bpy.utils.unregister_class(BezierExport)
