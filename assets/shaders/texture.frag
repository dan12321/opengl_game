#version 330 core
struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    int shininess;
};

struct PointLight {
    vec3 position;
    vec3 diffuse;
    vec3 specular;
    float strength;
};

in vec2 TexCoord;
in vec3 FragPos;
in vec3 Normal;

uniform sampler2D texture0;
uniform vec3 cameraPosition;
uniform Material material;

#define NR_POINT_LIGHTS 64
uniform PointLight pointLights[NR_POINT_LIGHTS];

out vec4 FragColor;

vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 cameraDir);
void main()
{
    vec4 tex = texture2D(texture0, TexCoord);
    vec3 cameraDir = normalize(cameraPosition - FragPos);

    vec3 color = material.ambient;
    for (int i = 0; i < NR_POINT_LIGHTS; i++) {
        color += CalcPointLight(pointLights[i], Normal, FragPos, cameraDir);
    }

    FragColor = vec4(color, 1.0) * tex;
}

vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 cameraDir)
{
    vec3 lightDir = normalize(light.position - fragPos);
    float lightDist = distance(fragPos, light.position);
    float distanceFallOff = min(1.0, light.strength / pow(lightDist, 2.0));

    float diffuseIntensity = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = distanceFallOff * diffuseIntensity * material.diffuse * light.diffuse;

    vec3 reflectDir = reflect(-lightDir, normal);
    float specularIntensity = distanceFallOff * pow(max(dot(cameraDir, reflectDir), 0.0), material.shininess);
    vec3 specular = specularIntensity * light.specular * material.specular;

    return specular + diffuse;
}
