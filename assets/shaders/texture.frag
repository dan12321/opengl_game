#version 330 core
struct Material {
    vec3 ambient;
    vec3 diffuse;
    sampler2D specular;
    int shininess;
};

struct PointLight {
    vec3 position;
    vec3 diffuse;
    vec3 specular;
    float strength;
};

struct DirLight {
    vec3 direction;
    vec3 diffuse;
    vec3 specular;
};

in vec2 TexCoord;
in vec3 FragPos;
in vec3 Normal;

uniform sampler2D texture0;
uniform vec3 cameraPosition;
uniform Material material;
uniform float offset;

#define NR_POINT_LIGHTS 64
#define NR_DIR_LIGHTS 64
uniform PointLight pointLights[NR_POINT_LIGHTS];
uniform DirLight dirLights[NR_DIR_LIGHTS];

out vec4 FragColor;

vec3 CalcPointLight(PointLight light, vec2 texCoord, vec3 normal, vec3 fragPos, vec3 cameraDir);
vec3 CalcDirLight(DirLight light, vec2 texCoord, vec3 normal, vec3 cameraDir);
void main()
{
    vec2 texCoord = vec2(TexCoord.x, TexCoord.y + offset);
    vec4 tex = texture2D(texture0, texCoord);
    vec3 cameraDir = normalize(cameraPosition - FragPos);

    vec3 color = material.ambient;
    for (int i = 0; i < NR_POINT_LIGHTS; i++) {
        color += CalcPointLight(pointLights[i], texCoord, Normal, FragPos, cameraDir);
    }

    for (int i = 0; i < NR_DIR_LIGHTS; i++) {
        color += CalcDirLight(dirLights[i], texCoord, Normal, cameraDir);
    }

    FragColor = vec4(color, 1.0) * tex;
}

vec3 CalcPointLight(PointLight light, vec2 texCoord, vec3 normal, vec3 fragPos, vec3 cameraDir)
{
    vec3 lightDir = normalize(light.position - fragPos);
    float lightDist = distance(fragPos, light.position);
    float distanceFallOff = min(1.0, light.strength / pow(lightDist, 2.0));

    float diffuseIntensity = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = distanceFallOff * diffuseIntensity * material.diffuse * light.diffuse;

    vec3 reflectDir = reflect(-lightDir, normal);
    float specularIntensity = distanceFallOff * min(1.0, pow(max(dot(cameraDir, reflectDir), 0.0), material.shininess));
    vec3 specular = specularIntensity * light.specular * texture2D(material.specular, texCoord).rgb;

    return specular + diffuse;
}

vec3 CalcDirLight(DirLight light, vec2 texCoord, vec3 normal, vec3 cameraDir)
{
    float diffuseIntensity = max(dot(normal, light.direction), 0.0);
    vec3 diffuse = diffuseIntensity * material.diffuse * light.diffuse;

    vec3 reflectDir = reflect(-light.direction, normal);
    float specularIntensity = min(pow(max(dot(cameraDir, reflectDir), 0.0), material.shininess), 1);
    vec3 specular = specularIntensity * light.specular * texture2D(material.specular, texCoord).rgb;

    return specular + diffuse;
}
