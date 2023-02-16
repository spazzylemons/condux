bl_info = {
    'name': 'Condux Tools',
    'author': 'spazzylemons',
    'version': (1, 0, 0),
    'blender': (3, 4, 1),
    'location': 'View3D > Object',
    'description': 'Tools for exporting data for Condux',
    'category': 'Import-Export',
}

import bpy, bpy_extras, math, struct, os

def write_co(file, co):
    for scalar in co.xzy:
        v = int(scalar * 256)
        file.write(struct.pack('<h', v))

class BezierExport(bpy.types.Operator, bpy_extras.io_utils.ExportHelper):
    """Export the selected Bezier curve."""
    bl_idname = 'condux.bezier_export'
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
                # iterate over the points in the curve - export them as 8.8
                for point in points:
                    write_co(file, point.co)
                    file.write(bytes([int(256 * ((point.tilt % (2 * math.pi)) / (2 * math.pi)))]))
        except BaseException as e:
            self.report({'ERROR'}, repr(e))
            return {'CANCELLED'}
        else:
            return {'FINISHED'}

class WireframeExport(bpy.types.Operator, bpy_extras.io_utils.ExportHelper):
    """Export the selected wireframe mesh."""
    bl_idname = 'condux.wireframe_export'
    bl_label = 'Export Wireframe Mesh'

    filename_ext = '.bin'

    def execute(self, context):
        try:
            # get the active mesh
            mesh = bpy.data.meshes[context.active_object.data.name]
            # get the edges
            edges = []
            for edge in mesh.edges:
                edges.append(tuple(i for i in edge.vertices))
            # save to file
            with open(self.filepath, 'wb') as file:
                # write number of points
                file.write(bytes([len(mesh.vertices)]))
                for vertex in mesh.vertices:
                    write_co(file, vertex.co)
                # write number of edges
                file.write(bytes([len(edges)]))
                for edge in edges:
                    # write endpoints
                    file.write(bytes(edge))
        except BaseException as e:
            self.report({'ERROR'}, repr(e))
            return {'CANCELLED'}
        else:
            return {'FINISHED'}

class FontExport(bpy.types.Operator, bpy_extras.io_utils.ExportHelper):
    """Export the selected font meshes."""
    bl_idname = 'condux.font_export'
    bl_label = 'Export Font Meshes'

    filename_ext = '.bin'

    def execute(self, context):
        try:
            with open(self.filepath, 'wb') as file:
                # find glyphs
                for charcode in range(0x20, 0x7f):
                    name = '{:02x}'.format(charcode)
                    if name not in bpy.data.objects:
                        raise RuntimeError('no glyph for {}'.format(repr(chr(charcode))))
                    # get the mesh
                    mesh = bpy.data.meshes[bpy.data.objects[name].data.name]
                    # get the edges
                    edges = []
                    for edge in mesh.edges:
                        edges.append(tuple(i for i in edge.vertices))
                    # write data to the file
                    v = len(mesh.vertices)
                    e = len(mesh.edges)
                    assert 0 <= v <= 15
                    assert 0 <= e <= 15
                    # write number of points
                    file.write(bytes([v | (e << 4)]))
                    for vertex in mesh.vertices:
                        # write as integers
                        x = round(vertex.co.x)
                        y = round(vertex.co.y)
                        assert 0 <= x <= 15
                        assert 0 <= y <= 15
                        file.write(bytes([x | (y << 4)]))
                    for i, j in edges:
                        # write endpoints
                        assert 0 <= i <= 15
                        assert 0 <= j <= 15
                        file.write(bytes([i | (j << 4)]))
        except BaseException as e:
            self.report({'ERROR'}, repr(e))
            return {'CANCELLED'}
        else:
            return {'FINISHED'}

def bezier_func(self, context):
    self.layout.operator(BezierExport.bl_idname)

def wireframe_func(self, context):
    self.layout.operator(WireframeExport.bl_idname)

def font_func(self, context):
    self.layout.operator(FontExport.bl_idname)

def register():
    bpy.utils.register_class(BezierExport)
    bpy.types.VIEW3D_MT_object.append(bezier_func)
    bpy.utils.register_class(WireframeExport)
    bpy.types.VIEW3D_MT_object.append(wireframe_func)
    bpy.utils.register_class(FontExport)
    bpy.types.VIEW3D_MT_object.append(font_func)

def unregister():
    bpy.types.VIEW3D_MT_object.remove(font_func)
    bpy.utils.unregister_class(FontExport)
    bpy.types.VIEW3D_MT_object.remove(wireframe_func)
    bpy.utils.unregister_class(WireframeExport)
    bpy.types.VIEW3D_MT_object.remove(bezier_func)
    bpy.utils.unregister_class(BezierExport)
