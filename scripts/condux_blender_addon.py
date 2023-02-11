bl_info = {
    'name': 'Condux Tools',
    'author': 'spazzylemons',
    'version': (1, 0, 0),
    'blender': (3, 4, 1),
    'location': 'View3D > Object',
    'description': 'Tools for exporting data for Condux',
    'category': 'Import-Export',
}

import bpy, bpy_extras, math, struct


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

def approximate_best_mesh(remaining):
    if not len(remaining):
        return ()

    strips = set()
    for edge in remaining:
        p, q = edge
        strips.add(((edge,), (p, q)))
        strips.add(((edge,), (q, p)))

    available_edges = set(remaining)
    replenished = True

    while True:
        new_strips = set()
        for used, strip in strips:
            p = strip[-1]
            while True:
                for edge in available_edges:
                    if edge not in used and p in edge:
                        q = next(i for i in edge if i != p)
                        new_strips.add((used + (edge,), strip + (q,)))
                        break
                else:
                    break
                available_edges.remove(edge)
        if len(new_strips):
            strips = new_strips
            replenished = False
        elif replenished:
            break
        else:
            available_edges = set(remaining)
            replenished = True

    result = None

    for used, strip in strips:
        new_remaining = set(remaining)
        for edge in used:
            new_remaining.remove(edge)
        x = (strip,) + approximate_best_mesh(tuple(new_remaining))
        if result is None or len(x) < len(result):
            result = x

    return result

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
            strips = approximate_best_mesh(edges)
            # save to file
            with open(self.filepath, 'wb') as file:
                # write number of points
                file.write(bytes([len(mesh.vertices)]))
                for vertex in mesh.vertices:
                    write_co(file, vertex.co)
                # write number of points
                file.write(bytes([len(strips)]))
                for strip in strips:
                    # write length of strip
                    file.write(bytes([len(strip)]))
                    # write strip
                    file.write(bytes(strip))
        except BaseException as e:
            self.report({'ERROR'}, repr(e))
            return {'CANCELLED'}
        else:
            return {'FINISHED'}

def bezier_func(self, context):
    self.layout.operator(BezierExport.bl_idname)

def wireframe_func(self, context):
    self.layout.operator(WireframeExport.bl_idname)

def register():
    bpy.utils.register_class(BezierExport)
    bpy.types.VIEW3D_MT_object.append(bezier_func)
    bpy.utils.register_class(WireframeExport)
    bpy.types.VIEW3D_MT_object.append(wireframe_func)

def unregister():
    bpy.types.VIEW3D_MT_object.remove(wireframe_func)
    bpy.utils.unregister_class(WireframeExport)
    bpy.types.VIEW3D_MT_object.remove(bezier_func)
    bpy.utils.unregister_class(BezierExport)
